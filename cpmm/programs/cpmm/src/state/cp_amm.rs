use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::utils::Q64_64;
use crate::error::ErrorCode;
use crate::state::AmmsConfig;

#[account]
#[derive(InitSpace)]
pub struct CpAmm {
    is_launched: bool, // 1
    // Base liquidity that will be locked forever after pool launch
    // Used for stabilizing pool if empty
    initial_locked_base_liquidity: u64, // 8
    // Quote liquidity that will be locked forever after pool launch
    // Used for stabilizing pool if empty
    initial_locked_quote_liquidity: u64, // 8
    
    // Square root of the constant product of the pool
    // Stored as square root in Q64.64 for computation accuracy 
    sqrt_constant_product: Q64_64, // 16
    // Square root of the Base and Quote token's ration
    // Stored as square root in Q64.64 for computation accuracy 
    base_quote_ratio: Q64_64, // 16
    
    // Base token amount in pool's vault
    base_liquidity: u64,   // 8
    // Quote token amount in pool's vault
    quote_liquidity: u64,  // 8
    // Amount of lp tokens minted to liquidity providers
    lp_tokens_supply: u64, // 8
    
    // Providers fee rate in basis points set by pool creator (1 = 0.01%)
    providers_fee_rate_basis_points: u16, // 2
    // Protocol fee from bound AmmsConfig account (1 = 0.01%)
    protocol_fee_rate_basis_points: u16, // 2
    
    // Base token fees to redeem by bound AmmsConfig account's authority 
    protocol_base_fees_to_redeem: u64,  // 8
    // Quote token fees to redeem by bound AmmsConfig account's authority 
    protocol_quote_fees_to_redeem: u64, // 8
    
    // Mint of the base token
    base_mint: Pubkey,  // 32
    // Mint of the quote token
    quote_mint: Pubkey, // 32
    // Mint of the liquidity token
    lp_mint: Pubkey,    // 32
    
    // Liquidity vault with base tokens
    base_vault: Pubkey, // 32
    // Liquidity vault with quote tokens
    quote_vault: Pubkey, // 32
    
    // AmmsConfig account
    amms_config: Pubkey, // 32
    // Canonical bump
    bump: u8, // 1
}

impl CpAmm {
    pub const SEED: &'static[u8] = b"cp_amm";
    const SWAP_CONSTANT_PRODUCT_TOLERANCE: f64 = 0.000001;
    const ADJUST_LIQUIDITY_RATIO_TOLERANCE: f64 = 0.000001;
    pub fn swap_base_to_quote_amount(&self, base_amount: u64, estimated_quote_amount: u64, allowed_slippage: u64) -> Result<SwapResult>{
        require!(self.quote_liquidity > 0, ErrorCode::QuoteLiquidityIsZero);
        require!(self.base_liquidity > 0, ErrorCode::BaseLiquidityIsZero);
        require!(base_amount > 0, ErrorCode::SwapAmountIsZero);

        let base_fee_amount = self.calculate_providers_fee_amount(base_amount);
        let base_protocol_fee_amount = self.calculate_protocol_fee_amount(base_amount);
        let base_amount_after_fees = base_amount.checked_sub(base_fee_amount).unwrap().checked_sub(base_protocol_fee_amount).unwrap();

        let new_base_liquidity =self.base_liquidity.checked_add(base_amount_after_fees).unwrap() ;
        let new_quote_liquidity = (self.sqrt_constant_product / Q64_64::sqrt_from_u128(new_base_liquidity as u128)).square_as_u64();

        let quote_delta = self.quote_liquidity.checked_sub(new_quote_liquidity).unwrap();

        require!(quote_delta.abs_diff(estimated_quote_amount) <= allowed_slippage, ErrorCode::SwapSlippageExceeded);
        self.check_constant_product_after_swap(new_base_liquidity, new_quote_liquidity)?;

        Ok(SwapResult::new(
            base_fee_amount + base_amount_after_fees,
            base_protocol_fee_amount,
            quote_delta,
            true
        ))
    }
    pub fn swap_quote_to_base_amount(&self, quote_amount: u64, estimated_base_amount: u64, allowed_slippage: u64) -> Result<SwapResult>{
        require!(self.quote_liquidity > 0, ErrorCode::QuoteLiquidityIsZero);
        require!(self.base_liquidity > 0, ErrorCode::BaseLiquidityIsZero);
        require!(quote_amount > 0, ErrorCode::SwapAmountIsZero);

        let quote_fee_amount = self.calculate_providers_fee_amount(quote_amount);
        let quote_protocol_fee_amount = self.calculate_protocol_fee_amount(quote_amount);
        let quote_amount_after_fees = quote_amount.checked_sub(quote_fee_amount).unwrap().checked_sub(quote_protocol_fee_amount).unwrap();

        let new_quote_liquidity = self.quote_liquidity.checked_add(quote_amount_after_fees).unwrap();
        let new_base_liquidity = (self.sqrt_constant_product / Q64_64::sqrt_from_u128(new_quote_liquidity as u128)).square_as_u64();

        let base_delta = self.base_liquidity.checked_sub(new_base_liquidity).unwrap();

        require!(base_delta.abs_diff(estimated_base_amount) <= allowed_slippage, ErrorCode::SwapSlippageExceeded);
        self.check_constant_product_after_swap(new_base_liquidity, new_quote_liquidity)?;

        Ok(SwapResult::new(
            quote_fee_amount + quote_amount_after_fees,
            quote_protocol_fee_amount,
            base_delta,
            false
        ))
    }
    pub fn update_after_swap(&mut self, swap_result: SwapResult) -> Result<()> {
        if swap_result.is_in_out(){
            self.base_liquidity = self.base_liquidity.checked_add(swap_result.in_amount_to_add()).ok_or(ErrorCode::UpdateAfterSwapOverflow)?;
            self.quote_liquidity = self.quote_liquidity.checked_sub(swap_result.out_amount_to_withdraw()).ok_or(ErrorCode::UpdateAfterSwapOverflow)?;
            self.protocol_base_fees_to_redeem = self.protocol_base_fees_to_redeem.checked_add(swap_result.in_protocol_fee).ok_or(ErrorCode::UpdateAfterSwapOverflow)?;
        }
        else{
            self.quote_liquidity = self.quote_liquidity.checked_add(swap_result.in_amount_to_add()).ok_or(ErrorCode::UpdateAfterSwapOverflow)?;
            self.base_liquidity = self.base_liquidity.checked_sub(swap_result.out_amount_to_withdraw()).ok_or(ErrorCode::UpdateAfterSwapOverflow)?;
            self.protocol_quote_fees_to_redeem = self.protocol_quote_fees_to_redeem.checked_add(swap_result.in_protocol_fee).ok_or(ErrorCode::UpdateAfterSwapOverflow)?;
        }
        self.sqrt_constant_product = self.calculate_sqrt_constant_product();
        self.base_quote_ratio = self.calculate_base_quote_ratio();
        Ok(())
    }
    fn check_constant_product_after_swap(&self, new_base_liquidity: u64, new_quote_liquidity: u64) -> Result<()>{
        let new_sqrt_constant_product = Q64_64::sqrt_from_u128(new_base_liquidity as u128 * new_quote_liquidity as u128);
        let difference = self.sqrt_constant_product.abs_diff(new_sqrt_constant_product);
        let allowed_difference = self.sqrt_constant_product * Q64_64::from_f64(Self::SWAP_CONSTANT_PRODUCT_TOLERANCE);
        require!(difference <= allowed_difference, ErrorCode::SwapConstantProductToleranceExceeded);
        Ok(())
    }
    fn check_ratio_after_liquidity_adjust(&self, new_base_liquidity: u64, new_quote_liquidity: u64) -> Result<()>{
        let new_base_quote_ratio = Q64_64::from_u64(new_base_liquidity) / Q64_64::from_u64(new_quote_liquidity);
        let difference = self.base_quote_ratio.abs_diff(new_base_quote_ratio);
        let allowed_difference = self.base_quote_ratio * Q64_64::from_f64(Self::ADJUST_LIQUIDITY_RATIO_TOLERANCE);
        require!(difference <= allowed_difference, ErrorCode::AdjustLiquidityRatioToleranceExceeded);
        Ok(())
    }
    pub fn calculate_base_quote_ratio(&self) -> Q64_64{
        Q64_64::from_u64(self.base_liquidity) / Q64_64::from_u64(self.quote_liquidity)
    }
    pub fn calculate_sqrt_constant_product(&self) -> Q64_64{
        Q64_64::sqrt_from_u128(self.base_liquidity as u128 * self.quote_liquidity as u128)
    }
    pub fn calculate_protocol_fee_amount(&self, swap_amount: u64) -> u64{
        ((swap_amount as u128).checked_mul(self.protocol_fee_rate_basis_points as u128).unwrap() / 10000u128) as u64
    }
    pub fn calculate_providers_fee_amount(&self, swap_amount: u64) -> u64{
        ((swap_amount as u128).checked_mul(self.providers_fee_rate_basis_points as u128).unwrap() / 10000u128) as u64
    }
    
    pub fn initialize(
        &mut self,
        base_mint: &Account<Mint>, 
        quote_mint: &Account<Mint>, 
        lp_mint: &Account<Mint>,
        amms_config: &Account<AmmsConfig>,
        bump: u8,
    ) -> (){
        
        self.providers_fee_rate_basis_points = amms_config.providers_fee_rate_basis_points;
        self.protocol_fee_rate_basis_points = amms_config.protocol_fee_rate_basis_points;
        
        self.base_mint = base_mint.key();
        self.quote_mint = quote_mint.key();
        self.lp_mint = lp_mint.key();
        self.amms_config = amms_config.key();
        self.is_launched = false;
        self.bump = bump;
    }
}

pub struct SwapResult{
    in_amount_to_add: u64,
    in_protocol_fee: u64,
    out_amount_to_withdraw: u64,
    is_in_out: bool,
}
impl SwapResult{
    fn new(in_amount_to_add: u64, in_protocol_fee: u64, out_amount_to_withdraw: u64, is_in_out: bool) -> Self {
        Self{
            in_amount_to_add,
            in_protocol_fee,
            out_amount_to_withdraw,
            is_in_out,
        }
    }
    pub fn in_amount_to_add(&self) -> u64{
        self.in_amount_to_add
    }
    pub fn in_protocol_fee(&self) -> u64{
        self.in_protocol_fee
    }
    pub fn out_amount_to_withdraw(&self) -> u64{
        self.out_amount_to_withdraw
    }
    pub fn is_in_out(&self) -> bool{
        self.is_in_out
    }
}