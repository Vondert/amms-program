use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use anchor_spl::token_interface;
use crate::utils::math::Q64_64;
use crate::error::ErrorCode;
use crate::state::AmmsConfig;
use super::CpAmmCalculate;

#[account]
#[derive(InitSpace)]
pub struct CpAmm {
    is_initialized: bool, // 1
    is_launched: bool, // 1
    
    // Liquidity that will be locked forever after pool launch
    // Used for stabilizing pool if empty
    initial_locked_liquidity: u64, // 8
    
    // Square root of the constant product of the pool
    // Stored as square root in Q64.64 for computation accuracy 
    constant_product_sqrt: Q64_64, // 16
    // Base and Quote token's ration
    // Stored as Q64.64 for computation accuracy 
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
    // Vault with locked liquidity tokens
    locked_lp_vault: Pubkey, // 32
    
    // AmmsConfig account
    amms_config: Pubkey, // 32
    // Canonical bump
    bump: [u8; 1], // 1
}

impl CpAmm {
    pub const SEED: &'static [u8] = b"cp_amm";
    pub fn seeds(&self) -> [&[u8]; 3] {
        [Self::SEED, self.lp_mint.as_ref(), self.bump.as_ref()]
    }
    pub fn is_initialized(&self) -> bool{
        self.is_initialized
    }
    pub fn is_launched(&self) -> bool {
        self.is_launched
    }
    pub fn bump(&self) -> u8 {
        self.bump[0]
    }
    pub fn base_mint(&self) -> &Pubkey{
        &self.base_mint
    }

    pub fn quote_mint(&self) -> &Pubkey{
        &self.quote_mint
    }
    pub fn lp_mint(&self) -> &Pubkey{
        &self.lp_mint
    }
    pub fn base_vault(&self) -> &Pubkey{
        &self.base_vault
    }
    pub fn quote_vault(&self) -> &Pubkey{
        &self.quote_vault
    }
    pub fn amms_config(&self) -> &Pubkey{
        &self.amms_config
    }
}
impl CpAmmCalculate for CpAmm {
    fn constant_product_sqrt(&self) -> Q64_64 {
        self.constant_product_sqrt
    }

    fn base_quote_ratio(&self) -> Q64_64 {
        self.base_quote_ratio
    }

    fn base_liquidity(&self) -> u64 {
        self.base_liquidity
    }

    fn quote_liquidity(&self) -> u64 {
        self.quote_liquidity
    }

    fn lp_tokens_supply(&self) -> u64 {
        self.lp_tokens_supply
    }

    fn providers_fee_rate_basis_points(&self) -> u16 {
        self.providers_fee_rate_basis_points
    }

    fn protocol_fee_rate_basis_points(&self) -> u16 {
        self.protocol_fee_rate_basis_points
    }
}

impl CpAmm {
    fn check_state(&self) -> Result<()>{
        require!(self.is_launched, ErrorCode::CpAmmNotLaunched);
        require!(self.quote_liquidity > 0, ErrorCode::QuoteLiquidityIsZero);
        require!(self.base_liquidity > 0, ErrorCode::BaseLiquidityIsZero);
        require!(self.lp_tokens_supply > 0, ErrorCode::LpTokensSupplyIsZero);
        Ok(())
    }
    pub fn get_launch_payload(&self, base_liquidity: u64, quote_liquidity: u64) -> Result<LaunchPayload> {
        require!(!self.is_launched, ErrorCode::CpAmmAlreadyLaunched);
        require!(self.is_initialized, ErrorCode::CpAmmNotInitialized);
        require!(base_liquidity > 0, ErrorCode::ProvidedBaseLiquidityIsZero);
        require!(quote_liquidity > 0, ErrorCode::ProvidedQuoteLiquidityIsZero);

        let constant_product_sqrt = Self::calculate_constant_product_sqrt(base_liquidity, quote_liquidity).unwrap();
        let (lp_tokens_supply, initial_locked_liquidity) = Self::calculate_launch_lp_tokens(constant_product_sqrt)?;
        let base_quote_ratio = Self::calculate_base_quote_ratio(base_liquidity, quote_liquidity).unwrap();
        
        Ok(LaunchPayload {
            initial_locked_liquidity,
            base_liquidity,
            quote_liquidity,
            constant_product_sqrt,
            base_quote_ratio,
            lp_tokens_supply,
        })
    }
    pub fn get_provide_payload(&self, base_liquidity: u64, quote_liquidity: u64) -> Result<ProvidePayload> {
        self.check_state()?;
        require!(base_liquidity > 0, ErrorCode::ProvidedBaseLiquidityIsZero);
        require!(quote_liquidity > 0, ErrorCode::ProvidedQuoteLiquidityIsZero);

        let new_base_liquidity = self.base_liquidity.checked_add(base_liquidity).ok_or(ErrorCode::ProvideOverflowError)?;
        let new_quote_liquidity = self.quote_liquidity.checked_add(quote_liquidity).ok_or(ErrorCode::ProvideOverflowError)?;
        let new_base_quote_ratio_sqrt =  self.validate_and_calculate_liquidity_ratio(new_base_liquidity, new_quote_liquidity)?;

        let new_constant_product_sqrt = Self::calculate_constant_product_sqrt(new_base_liquidity, new_quote_liquidity).unwrap();
        
        let lp_tokens_to_mint = self.calculate_lp_mint_for_provided_liquidity(new_constant_product_sqrt).ok_or(ErrorCode::LpTokensCalculationFailed)?;

        let new_lp_tokens_supply = self.lp_tokens_supply.checked_add(lp_tokens_to_mint).ok_or(ErrorCode::ProvideOverflowError)?;
        Ok(ProvidePayload {
            base_quote_ratio: new_base_quote_ratio_sqrt,
            constant_product: new_constant_product_sqrt,
            base_liquidity: new_base_liquidity,
            quote_liquidity: new_quote_liquidity,
            lp_tokens_supply: new_lp_tokens_supply,
            lp_tokens_to_mint,
        })
    }
    pub fn get_withdraw_payload(&self, lp_tokens: u64) -> Result<WithdrawPayload> {
        self.check_state()?;
        require!(lp_tokens > 0, ErrorCode::ProvidedLpTokensIsZero);

        let lp_tokens_left_supply = self.lp_tokens_supply.checked_sub(lp_tokens).ok_or(ErrorCode::WithdrawOverflowError)?;

        let (base_withdraw, quote_withdraw) = self.calculate_liquidity_from_share(lp_tokens).ok_or(ErrorCode::WithdrawLiquidityCalculationFailed)?;
        
        let new_base_liquidity = self.base_liquidity.checked_sub(base_withdraw).ok_or(ErrorCode::WithdrawOverflowError)?;
        let new_quote_liquidity = self.quote_liquidity.checked_sub(quote_withdraw).ok_or(ErrorCode::WithdrawOverflowError)?;

        // Checks that new base and quote liquidity don't equal zero and amm won't be drained
        let new_base_quote_ratio = self.validate_and_calculate_liquidity_ratio(new_base_liquidity, new_quote_liquidity)?;

        Ok(WithdrawPayload{
            base_quote_ratio: new_base_quote_ratio,
            base_liquidity: new_base_liquidity,
            quote_liquidity: new_quote_liquidity,
            lp_tokens_supply: lp_tokens_left_supply,
            base_withdraw_amount: base_withdraw,
            quote_withdraw_amount: quote_withdraw,
        })
    }
    pub fn get_base_to_quote_swap_payload(&self, base_amount: u64, estimated_result: u64, allowed_slippage: u64) -> Result<SwapPayload>{
        self.check_state()?;
        require!(base_amount > 0, ErrorCode::SwapAmountIsZero);
        require!(estimated_result > 0, ErrorCode::EstimatedResultIsZero);

        let base_fee_amount = self.calculate_providers_fee_amount(base_amount);
        let base_protocol_fee_amount = self.calculate_protocol_fee_amount(base_amount);
        let protocol_fees_to_redeem = self.protocol_base_fees_to_redeem.checked_add(base_protocol_fee_amount).ok_or(ErrorCode::SwapOverflowError)?;
        
        let base_amount_after_fees = base_amount.checked_sub(base_fee_amount).unwrap().checked_sub(base_protocol_fee_amount).ok_or(ErrorCode::SwapOverflowError)?;
        
        let (new_base_liquidity, new_quote_liquidity) = self.calculate_afterswap_liquidity(base_amount_after_fees, true).ok_or(ErrorCode::AfterswapCalculationFailed)?;

        // Check constant product change is in acceptable range
        self.validate_swap_constant_product(new_base_liquidity, new_quote_liquidity)?;

        let quote_delta = self.quote_liquidity.checked_sub(new_quote_liquidity).ok_or(ErrorCode::SwapOverflowError)?;

        Self::check_swap_result(quote_delta, estimated_result, allowed_slippage)?;

        Ok(SwapPayload::new(
            new_base_liquidity + base_fee_amount,
            new_quote_liquidity,
            protocol_fees_to_redeem,
            quote_delta,
            true
        ))
    }
    pub fn get_quote_to_base_swap_payload(&self, quote_amount: u64, estimated_result: u64, allowed_slippage: u64) -> Result<SwapPayload>{
        self.check_state()?;
        require!(quote_amount > 0, ErrorCode::SwapAmountIsZero);
        require!(estimated_result > 0, ErrorCode::EstimatedResultIsZero);

        let quote_fee_amount = self.calculate_providers_fee_amount(quote_amount);
        let quote_protocol_fee_amount = self.calculate_protocol_fee_amount(quote_amount);
        let protocol_fees_to_redeem = self.protocol_quote_fees_to_redeem.checked_add(quote_protocol_fee_amount).ok_or(ErrorCode::SwapOverflowError)?;
        
        let quote_amount_after_fees = quote_amount.checked_sub(quote_fee_amount).unwrap().checked_sub(quote_protocol_fee_amount).ok_or(ErrorCode::SwapOverflowError)?;

        let (new_base_liquidity, new_quote_liquidity) = self.calculate_afterswap_liquidity(quote_amount_after_fees, false).ok_or(ErrorCode::AfterswapCalculationFailed)?;

        // Check constant product change is in acceptable range
        self.validate_swap_constant_product(new_base_liquidity, new_quote_liquidity)?;

        let base_delta = self.base_liquidity.checked_sub(new_base_liquidity).ok_or(ErrorCode::SwapOverflowError)?;

        Self::check_swap_result(base_delta, estimated_result, allowed_slippage)?;

        Ok(SwapPayload::new(
            new_base_liquidity,
            new_quote_liquidity + quote_fee_amount,
            protocol_fees_to_redeem,
            base_delta,
            false
        ))
    }
}

impl CpAmm {
    pub fn initialize(
        &mut self,
        base_mint: &InterfaceAccount<token_interface::Mint>,
        quote_mint: &InterfaceAccount<token_interface::Mint>,
        lp_mint: &Account<Mint>,
        amms_config: &Account<AmmsConfig>,
        bump: u8,
    ) -> Result<()>{
        require!(!self.is_initialized, ErrorCode::CpAmmAlreadyInitialized);

        self.providers_fee_rate_basis_points = amms_config.providers_fee_rate_basis_points;
        self.protocol_fee_rate_basis_points = amms_config.protocol_fee_rate_basis_points;

        self.base_mint = base_mint.key();
        self.quote_mint = quote_mint.key();
        self.lp_mint = lp_mint.key();
        self.amms_config = amms_config.key();
        self.is_launched = false;
        self.is_initialized = true;
        self.bump = [bump];

        Ok(())
    }
    pub(crate) fn launch(&mut self, launch_payload: LaunchPayload, base_vault: &InterfaceAccount<token_interface::TokenAccount>, quote_vault: &InterfaceAccount<token_interface::TokenAccount>, locked_lp_vault: &Account<TokenAccount>) -> (){
        self.base_liquidity = launch_payload.base_liquidity;
        self.quote_liquidity = launch_payload.quote_liquidity;
        self.initial_locked_liquidity = launch_payload.initial_locked_liquidity;
        self.lp_tokens_supply = launch_payload.lp_tokens_supply;
        self.constant_product_sqrt = launch_payload.constant_product_sqrt;
        self.base_quote_ratio = launch_payload.base_quote_ratio;
        self.base_vault = base_vault.key();
        self.quote_vault = quote_vault.key();
        self.locked_lp_vault = locked_lp_vault.key();
    }
    pub(crate) fn provide(&mut self, provide_payload: ProvidePayload) -> (){
        self.base_liquidity = provide_payload.base_liquidity;
        self.quote_liquidity = provide_payload.quote_liquidity;
        self.lp_tokens_supply = provide_payload.lp_tokens_supply;
        self.constant_product_sqrt = provide_payload.constant_product;
        self.base_quote_ratio = provide_payload.base_quote_ratio;
    }
    pub(crate) fn withdraw(&mut self, withdraw_payload: WithdrawPayload) -> (){
        self.base_liquidity = withdraw_payload.base_liquidity;
        self.quote_liquidity = withdraw_payload.quote_liquidity;
        self.lp_tokens_supply = withdraw_payload.lp_tokens_supply;
        self.constant_product_sqrt = Self::calculate_constant_product_sqrt(self.base_liquidity, self.quote_liquidity).unwrap();
        self.base_quote_ratio = withdraw_payload.base_quote_ratio;
    }
    pub(crate) fn swap(&mut self, swap_payload: SwapPayload) -> () {
        self.base_liquidity = swap_payload.base_liquidity;
        self.quote_liquidity = swap_payload.quote_liquidity;
        if swap_payload.is_in_out{
            self.protocol_base_fees_to_redeem = swap_payload.protocol_fees_to_redeem
        }
        else{
            self.protocol_quote_fees_to_redeem = swap_payload.protocol_fees_to_redeem;
        }
        self.constant_product_sqrt = Self::calculate_constant_product_sqrt(self.base_liquidity, self.quote_liquidity).unwrap();
        self.base_quote_ratio = Self::calculate_base_quote_ratio(self.base_liquidity, self.quote_liquidity).unwrap();
    }
}

#[derive(Debug)]
pub struct LaunchPayload {
    initial_locked_liquidity: u64,
    constant_product_sqrt: Q64_64,
    base_quote_ratio: Q64_64,
    base_liquidity: u64,
    quote_liquidity: u64,
    lp_tokens_supply: u64,
}
impl LaunchPayload {
    pub fn initial_locked_liquidity(&self) -> u64{
        self.initial_locked_liquidity
    }
    pub fn launch_liquidity(&self) -> u64{
        self.lp_tokens_supply.checked_sub(self.initial_locked_liquidity).unwrap()
    }
}

#[derive(Debug)]
pub struct ProvidePayload {
    base_quote_ratio: Q64_64,
    constant_product: Q64_64,
    base_liquidity: u64,
    quote_liquidity: u64,
    lp_tokens_supply: u64,
    lp_tokens_to_mint: u64,
}
impl ProvidePayload {
    pub fn lp_tokens_to_mint(&self) -> u64{
        self.lp_tokens_to_mint
    }
}

#[derive(Debug)]
pub struct WithdrawPayload{
    base_quote_ratio: Q64_64,
    base_liquidity: u64,
    quote_liquidity: u64,
    lp_tokens_supply: u64,
    base_withdraw_amount: u64,
    quote_withdraw_amount: u64
}
impl WithdrawPayload {
    pub fn base_withdraw_amount(&self) -> u64{
        self.base_withdraw_amount
    }
    pub fn quote_withdraw_amount(&self) -> u64{
        self.quote_withdraw_amount
    }
}

#[derive(Debug)]
pub struct SwapPayload {
    base_liquidity: u64,
    quote_liquidity: u64,
    protocol_fees_to_redeem: u64,
    amount_to_withdraw: u64,
    is_in_out: bool,
}

impl SwapPayload {
    fn new(base_liquidity: u64, quote_liquidity: u64, protocol_fees_to_redeem: u64, amount_to_withdraw: u64, is_in_out: bool) -> Self {
        Self{
            base_liquidity,
            quote_liquidity,
            protocol_fees_to_redeem,
            amount_to_withdraw,
            is_in_out,
        }
    }
    pub fn amount_to_withdraw(&self) -> u64{
        self.amount_to_withdraw
    }
}