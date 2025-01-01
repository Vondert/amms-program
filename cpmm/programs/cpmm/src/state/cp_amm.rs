use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use anchor_spl::token_interface;
use crate::utils::math::Q64_64;
use crate::error::ErrorCode;
use crate::state::AmmsConfig;

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
    // Square root of the Base and Quote token's ration
    // Stored as square root in Q64.64 for computation accuracy 
    base_quote_ratio_sqrt: Q64_64, // 16
    
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
    pub const LP_MINT_INITIAL_DECIMALS: u8 = 5;
    const SWAP_CONSTANT_PRODUCT_TOLERANCE: f64 = 0.000001;
    const ADJUST_LIQUIDITY_RATIO_TOLERANCE: f64 = 0.000001;
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
impl CpAmm {
    pub fn get_launch_payload(&self, base_liquidity: u64, quote_liquidity: u64) -> Result<LaunchPayload> {
        require!(!self.is_launched, ErrorCode::CpAmmAlreadyLaunched);
        require!(self.is_initialized, ErrorCode::CpAmmNotInitialized);
        require!(base_liquidity > 0, ErrorCode::ProvidedBaseLiquidityIsZero);
        require!(quote_liquidity > 0, ErrorCode::ProvidedQuoteLiquidityIsZero);

        let constant_product_sqrt = Q64_64::sqrt_from_u128(base_liquidity as u128 * quote_liquidity as u128);
        let base_quote_ratio_sqrt =  Q64_64::sqrt_from_u128((Q64_64::from_u64(base_liquidity) / Q64_64::from_u64(quote_liquidity)).raw_value());

        let lp_tokens_supply = constant_product_sqrt.to_u64();
        require!(lp_tokens_supply > 0, ErrorCode::LpTokensToMintIsZero);

        let initial_locked_liquidity = 10_u64.pow(Self::LP_MINT_INITIAL_DECIMALS as u32);

        let difference = lp_tokens_supply.checked_sub(initial_locked_liquidity).ok_or(ErrorCode::LaunchLiquidityTooSmall)?;
        require!(difference >= initial_locked_liquidity << 2, ErrorCode::LaunchLiquidityTooSmall);

        Ok(LaunchPayload {
            initial_locked_liquidity,
            base_liquidity,
            quote_liquidity,
            constant_product_sqrt,
            base_quote_ratio_sqrt,
            lp_tokens_supply,
        })
    }
    pub fn get_provide_payload(&self, base_liquidity: u64, quote_liquidity: u64) -> Result<ProvidePayload> {
        self.check_state()?;
        require!(base_liquidity > 0, ErrorCode::ProvidedBaseLiquidityIsZero);
        require!(quote_liquidity > 0, ErrorCode::ProvidedQuoteLiquidityIsZero);

        let new_base_liquidity = self.base_liquidity.checked_add(base_liquidity).unwrap();
        let new_quote_liquidity = self.quote_liquidity.checked_add(quote_liquidity).unwrap();
        let new_base_quote_ratio_sqrt =  self.calculate_and_validate_liquidity_ratio(new_base_liquidity, new_quote_liquidity)?;

        let new_constant_product_sqrt = Q64_64::sqrt_from_u128(new_base_liquidity as u128 * new_quote_liquidity as u128);
        // In valid amm new constant product is always bigger than current
        let provided_liquidity = new_constant_product_sqrt - self.constant_product_sqrt;
        // In valid amm constant product is never 0
        let share_from_current_liquidity = provided_liquidity / self.constant_product_sqrt;

        let lp_tokens_to_mint = (share_from_current_liquidity * Q64_64::from_u64(self.lp_tokens_supply)).to_u64();
        require!(lp_tokens_to_mint > 0, ErrorCode::LpTokensToMintIsZero);

        let new_lp_tokens_supply = self.lp_tokens_supply.checked_add(lp_tokens_to_mint).unwrap();
        Ok(ProvidePayload {
            base_quote_ratio_sqrt: new_base_quote_ratio_sqrt,
            constant_product_sqrt: new_constant_product_sqrt,
            base_liquidity: new_base_liquidity,
            quote_liquidity: new_quote_liquidity,
            lp_tokens_supply: new_lp_tokens_supply,
            lp_tokens_to_mint,
        })
    }
    pub fn get_withdraw_payload(&self, lp_tokens: u64) -> Result<WithdrawPayload> {
        self.check_state()?;
        require!(lp_tokens > 0, ErrorCode::ProvidedLpTokensIsZero);

        let lp_tokens_left_supply = self.lp_tokens_supply.checked_sub(lp_tokens).unwrap();

        require!(lp_tokens_left_supply > 0, ErrorCode::LpTokensLeftSupplyIsZero);

        // Liquidity share is always less than or equal 1 but not 0
        let liquidity_share = Q64_64::from_u64(lp_tokens) / Q64_64::from_u64(self.lp_tokens_supply);
        let constant_product_sqrt_share = self.constant_product_sqrt * liquidity_share;


        // x * y = k guarantees multiplication and division without overflow
        let base_withdraw = (constant_product_sqrt_share * self.base_quote_ratio_sqrt).to_u64();
        let quote_withdraw = (constant_product_sqrt_share / self.base_quote_ratio_sqrt).to_u64();

        require!(base_withdraw > 0, ErrorCode::BaseWithdrawAmountIsZero);
        require!(quote_withdraw > 0, ErrorCode::QuoteWithdrawAmountIsZero);

        let new_base_liquidity = self.base_liquidity.checked_sub(base_withdraw).unwrap();
        let new_quote_liquidity = self.quote_liquidity.checked_sub(quote_withdraw).unwrap();

        // Checks that new base and quote liquidity don't equal zero and amm won't be drained
        let new_base_quote_ratio_sqrt = self.calculate_and_validate_liquidity_ratio(new_base_liquidity, new_quote_liquidity)?;

        Ok(WithdrawPayload{
            base_quote_ratio_sqrt: new_base_quote_ratio_sqrt,
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
        let protocol_fees_to_redeem = self.protocol_base_fees_to_redeem.checked_add(base_protocol_fee_amount).unwrap();
        let base_amount_after_fees = base_amount.checked_sub(base_fee_amount).unwrap().checked_sub(base_protocol_fee_amount).unwrap();

        require!(base_amount_after_fees > 0, ErrorCode::PostfeeSwapAmountIsZero);

        let new_base_liquidity =self.base_liquidity.checked_add(base_amount_after_fees).unwrap() ;
        let new_quote_liquidity = (self.constant_product_sqrt / Q64_64::sqrt_from_u128(new_base_liquidity as u128)).square_as_u64();

        // Check constant product change is in acceptable range
        self.validate_swap_constant_product(new_base_liquidity, new_quote_liquidity)?;

        let quote_delta = self.quote_liquidity.checked_sub(new_quote_liquidity).unwrap();

        self.check_swap_result(quote_delta, estimated_result, allowed_slippage)?;

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
        let protocol_fees_to_redeem = self.protocol_quote_fees_to_redeem.checked_add(quote_protocol_fee_amount).unwrap();
        let quote_amount_after_fees = quote_amount.checked_sub(quote_fee_amount).unwrap().checked_sub(quote_protocol_fee_amount).unwrap();

        require!(quote_amount_after_fees > 0, ErrorCode::PostfeeSwapAmountIsZero);

        let new_quote_liquidity = self.quote_liquidity.checked_add(quote_amount_after_fees).unwrap();
        let new_base_liquidity = (self.constant_product_sqrt / Q64_64::sqrt_from_u128(new_quote_liquidity as u128)).square_as_u64();

        // Check constant product change is in acceptable range
        self.validate_swap_constant_product(new_base_liquidity, new_quote_liquidity)?;

        let base_delta = self.base_liquidity.checked_sub(new_base_liquidity).unwrap();

        self.check_swap_result(base_delta, estimated_result, allowed_slippage)?;

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
        self.base_quote_ratio_sqrt = launch_payload.base_quote_ratio_sqrt;
        self.base_vault = base_vault.key();
        self.quote_vault = quote_vault.key();
        self.locked_lp_vault = locked_lp_vault.key();
    }
    pub(crate) fn provide(&mut self, provide_payload: ProvidePayload) -> (){
        self.base_liquidity = provide_payload.base_liquidity;
        self.quote_liquidity = provide_payload.quote_liquidity;
        self.lp_tokens_supply = provide_payload.lp_tokens_supply;
        self.constant_product_sqrt = provide_payload.constant_product_sqrt;
        self.base_quote_ratio_sqrt = provide_payload.base_quote_ratio_sqrt;
    }
    pub(crate) fn withdraw(&mut self, withdraw_payload: WithdrawPayload) -> (){
        self.base_liquidity = withdraw_payload.base_liquidity;
        self.quote_liquidity = withdraw_payload.quote_liquidity;
        self.lp_tokens_supply = withdraw_payload.lp_tokens_supply;
        self.constant_product_sqrt = self.calculate_constant_product_sqrt();
        self.base_quote_ratio_sqrt = withdraw_payload.base_quote_ratio_sqrt;
    }
    pub(crate) fn swap(&mut self, swap_payload: SwapPayload) -> () {
        self.base_liquidity = swap_payload.base_liquidity;
        self.quote_liquidity = swap_payload.quote_liquidity;
        if swap_payload.is_in_out(){
            self.protocol_base_fees_to_redeem = swap_payload.protocol_fees_to_redeem
        }
        else{
            self.protocol_quote_fees_to_redeem = swap_payload.protocol_fees_to_redeem;
        }
        self.constant_product_sqrt = self.calculate_constant_product_sqrt();
        self.base_quote_ratio_sqrt = self.calculate_base_quote_ratio_sqrt();
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
    fn check_swap_result(&self, swap_result: u64, estimated_swap_result: u64, allowed_slippage:u64) -> Result<()>{
        require!(swap_result > 0, ErrorCode::SwapResultIsZero);
        require!(swap_result.abs_diff(estimated_swap_result) <= allowed_slippage, ErrorCode::SwapSlippageExceeded);
        Ok(())
    }
    fn calculate_and_validate_liquidity_ratio(&self, new_base_liquidity: u64, new_quote_liquidity: u64) -> Result<Q64_64>{
        require!(new_base_liquidity > 0, ErrorCode::NewBaseLiquidityIsZero);
        require!(new_quote_liquidity > 0, ErrorCode::NewQuoteLiquidityIsZero);
        let new_base_quote_ratio_sqrt = Q64_64::sqrt_from_u128((Q64_64::from_u64(new_base_liquidity) / Q64_64::from_u64(new_quote_liquidity)).raw_value());
        let difference = self.base_quote_ratio_sqrt.abs_diff(new_base_quote_ratio_sqrt);
        let allowed_difference = self.base_quote_ratio_sqrt * Q64_64::from_f64(Self::ADJUST_LIQUIDITY_RATIO_TOLERANCE);
        require!(difference <= allowed_difference, ErrorCode::LiquidityRatioToleranceExceeded);
        Ok(new_base_quote_ratio_sqrt)
    }
    fn validate_swap_constant_product(&self, new_base_liquidity: u64, new_quote_liquidity: u64) -> Result<()>{
        require!(new_base_liquidity > 0, ErrorCode::NewBaseLiquidityIsZero);
        require!(new_quote_liquidity > 0, ErrorCode::NewQuoteLiquidityIsZero);
        let new_constant_product_sqrt = Q64_64::sqrt_from_u128(new_base_liquidity as u128 * new_quote_liquidity as u128);
        let difference = self.constant_product_sqrt.abs_diff(new_constant_product_sqrt);
        let allowed_difference = self.constant_product_sqrt * Q64_64::from_f64(Self::SWAP_CONSTANT_PRODUCT_TOLERANCE);
        require!(difference <= allowed_difference, ErrorCode::ConstantProductToleranceExceeded);
        Ok(())
    }
    fn calculate_base_quote_ratio_sqrt(&self) -> Q64_64{
        Q64_64::sqrt_from_u128((Q64_64::from_u64(self.base_liquidity) / Q64_64::from_u64(self.quote_liquidity)).raw_value())
    }
    fn calculate_constant_product_sqrt(&self) -> Q64_64{
        Q64_64::sqrt_from_u128(self.base_liquidity as u128 * self.quote_liquidity as u128)
    }
    fn calculate_protocol_fee_amount(&self, swap_amount: u64) -> u64{
        ((swap_amount as u128).checked_mul(self.protocol_fee_rate_basis_points as u128).unwrap() / 10000u128) as u64
    }
    fn calculate_providers_fee_amount(&self, swap_amount: u64) -> u64{
        ((swap_amount as u128).checked_mul(self.providers_fee_rate_basis_points as u128).unwrap() / 10000u128) as u64
    }
}

#[derive(Debug)]
pub struct LaunchPayload {
    initial_locked_liquidity: u64,
    constant_product_sqrt: Q64_64,
    base_quote_ratio_sqrt: Q64_64,
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
    base_quote_ratio_sqrt: Q64_64,
    constant_product_sqrt: Q64_64,
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
    base_quote_ratio_sqrt: Q64_64,
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
    pub fn is_in_out(&self) -> bool{
        self.is_in_out
    }
}