use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use anchor_spl::token_interface;
use crate::utils::math::Q64_128;
use crate::error::ErrorCode;
use crate::state::AmmsConfig;
use super::{CpAmmCalculate, CpAmmCore};

/// Represents a Constant Product Automated Market Maker (AMM) pool.
///
/// The `CpAmm` struct encapsulates all the key parameters and state of the pool,
/// including liquidity, fees, tokens, and associated vaults.
///
/// The AMM maintains the constant product invariant, which ensures that
/// the product of the pool's base and quote liquidity remains constant during swaps.
#[account]
#[derive(InitSpace)]
pub struct CpAmm {
    /// Whether the AMM has been initialized.
    is_initialized: bool, // 1 byte

    /// Whether the AMM has been launched and is active.
    is_launched: bool, // 1 byte

    /// Initial liquidity that is permanently locked after the pool launch.
    /// This stabilizes the pool in case of empty liquidity.
    initial_locked_liquidity: u64, // 8 bytes

    /// Square root of the constant product of the pool, stored as a Q64.128 fixed-point number.
    /// This ensures high accuracy during calculations.
    constant_product_sqrt: Q64_128, // 16 bytes

    /// Square root of the ratio between the base and quote tokens, stored as a Q64.128 fixed-point number.
    base_quote_ratio_sqrt: Q64_128, // 16 bytes

    /// Amount of base tokens currently in the pool's vault.
    base_liquidity: u64, // 8 bytes

    /// Amount of quote tokens currently in the pool's vault.
    quote_liquidity: u64, // 8 bytes

    /// Total supply of LP tokens minted to liquidity providers.
    lp_tokens_supply: u64, // 8 bytes

    /// Accumulated base token fees that can be redeemed by the `AmmsConfig` account's authority.
    protocol_base_fees_to_redeem: u64, // 8 bytes

    /// Accumulated quote token fees that can be redeemed by the `AmmsConfig` account's authority.
    protocol_quote_fees_to_redeem: u64, // 8 bytes

    /// Public key of the base token's mint.
    base_mint: Pubkey, // 32 bytes

    /// Public key of the quote token's mint.
    quote_mint: Pubkey, // 32 bytes

    /// Public key of the LP token's mint.
    pub lp_mint: Pubkey, // 32 bytes

    /// Public key of the vault holding the base tokens.
    base_vault: Pubkey, // 32 bytes

    /// Public key of the vault holding the quote tokens.
    quote_vault: Pubkey, // 32 bytes

    /// Public key of the vault holding locked LP tokens.
    locked_lp_vault: Pubkey, // 32 bytes

    /// Public key of the associated `AmmsConfig` account.
    amms_config: Pubkey, // 32 bytes

    /// Canonical bump seed for the account's PDA.
    bump: [u8; 1], // 1 byte
}

impl CpAmm {
    /// Seed used for generating the PDA.
    pub const SEED: &'static [u8] = b"cp_amm";

    /// Returns the seeds for generating the PDA.
    ///
    /// The PDA is derived using the `SEED`, the `lp_mint`, and the `bump` value.
    pub fn seeds(&self) -> [&[u8]; 3] {
        [Self::SEED, self.lp_mint.as_ref(), self.bump.as_ref()]
    }

    /// Checks if the AMM has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Checks if the AMM has been launched and is active.
    #[inline]
    pub fn is_launched(&self) -> bool {
        self.is_launched
    }

    /// Returns the canonical bump value for the PDA.
    #[inline]
    pub fn bump(&self) -> u8 {
        self.bump[0]
    }

    /// Returns the public key of the base token's mint.
    #[inline]
    pub fn base_mint(&self) -> &Pubkey {
        &self.base_mint
    }

    /// Returns the public key of the quote token's mint.
    #[inline]
    pub fn quote_mint(&self) -> &Pubkey { &self.quote_mint }

    /// Returns the public key of the vault holding base tokens.
    #[inline]
    pub fn base_vault(&self) -> &Pubkey {
        &self.base_vault
    }

    /// Returns the public key of the vault holding quote tokens.
    #[inline]
    pub fn quote_vault(&self) -> &Pubkey {
        &self.quote_vault
    }

    /// Returns the public key of the associated `AmmsConfig` account.
    #[inline]
    pub fn amms_config(&self) -> &Pubkey {
        &self.amms_config
    }
}
/// Implements the `CpAmmCore` trait for the `CpAmm` struct.
///
/// This implementation auto implements 'CpAmmCalculate' trait that defines core logic and calculations for the constant product AMM,
/// including methods for computing liquidity ratios, validating swaps, and handling fees.
impl CpAmmCore for CpAmm {
    /// Retrieves the square root of the constant product of the pool.
    ///
    /// # Returns
    /// - A `Q64_128` value representing the square root of the constant product,
    ///   used to maintain the AMM's invariant during calculations.
    fn constant_product_sqrt(&self) -> Q64_128 {
        self.constant_product_sqrt
    }

    /// Retrieves the square root of the base-to-quote token liquidity ratio.
    ///
    /// # Returns
    /// - A `Q64_128` value representing the square root of the liquidity ratio
    ///   between the base and quote tokens in the pool.
    fn base_quote_ratio_sqrt(&self) -> Q64_128 {
        self.base_quote_ratio_sqrt
    }

    /// Retrieves the current amount of base token liquidity in the pool's vault.
    ///
    /// # Returns
    /// - A `u64` value representing the amount of base tokens available in the pool.
    fn base_liquidity(&self) -> u64 {
        self.base_liquidity
    }

    /// Retrieves the current amount of quote token liquidity in the pool's vault.
    ///
    /// # Returns
    /// - A `u64` value representing the amount of quote tokens available in the pool.
    fn quote_liquidity(&self) -> u64 {
        self.quote_liquidity
    }

    /// Retrieves the total supply of LP tokens minted by the pool.
    ///
    /// # Returns
    /// - A `u64` value representing the total number of LP tokens issued.
    fn lp_tokens_supply(&self) -> u64 {
        self.lp_tokens_supply
    }
}

impl CpAmm {
    /// Validates the current state of the AMM to ensure it is ready for operations.
    ///
    /// This method ensures that:
    /// - The AMM has been launched.
    /// - The pool has non-zero base and quote liquidity.
    /// - The pool has a positive supply of LP tokens.
    ///
    /// # Returns
    /// - `Ok(())` if the state is valid.
    /// - `Err(ErrorCode)` if any of the checks fail.
    #[inline]
    fn check_state(&self) -> Result<()> {
        require!(self.is_launched, ErrorCode::CpAmmNotLaunched);
        require!(self.quote_liquidity > 0, ErrorCode::BaseLiquidityIsZero);
        require!(self.base_liquidity > 0, ErrorCode::QuoteLiquidityIsZero);
        require!(self.lp_tokens_supply > 0, ErrorCode::LpTokensSupplyIsZero);
        Ok(())
    }
    
    /// Prepares the payload for launching the AMM with the provided base and quote liquidity.
    ///
    /// It calculates the initial constant product, liquidity ratios, and the total supply of LP tokens to mint.
    ///
    /// # Parameters
    /// - `base_liquidity`: The amount of base liquidity to add during the launch.
    /// - `quote_liquidity`: The amount of quote liquidity to add during the launch.
    ///
    /// # Returns
    /// - `Ok(LaunchPayload)` containing the calculated launch details.
    /// - `Err(ErrorCode)` if any preconditions fail or calculations encounter errors.
    pub fn get_launch_payload(&self, base_liquidity: u64, quote_liquidity: u64) -> Result<LaunchPayload> {
        require!(!self.is_launched, ErrorCode::CpAmmAlreadyLaunched);
        require!(self.is_initialized, ErrorCode::CpAmmNotInitialized);
        require!(base_liquidity > 0, ErrorCode::ProvidedBaseLiquidityIsZero);
        require!(quote_liquidity > 0, ErrorCode::ProvidedQuoteLiquidityIsZero);

        let constant_product_sqrt = Self::calculate_constant_product_sqrt(base_liquidity, quote_liquidity).unwrap();
        let (lp_tokens_supply, initial_locked_liquidity) = Self::calculate_launch_lp_tokens(constant_product_sqrt)?;
        let base_quote_ratio_sqrt = Self::calculate_base_quote_ratio_sqrt(base_liquidity, quote_liquidity).unwrap();
        
        Ok(LaunchPayload {
            initial_locked_liquidity,
            base_liquidity,
            quote_liquidity,
            constant_product_sqrt,
            base_quote_ratio_sqrt,
            lp_tokens_supply,
        })
    }

    /// Prepares the payload for adding liquidity to the AMM.
    ///
    /// It calculates the new pool state, including updated liquidity, constant product, and the number of LP tokens to mint.
    ///
    /// # Parameters
    /// - `base_liquidity`: The amount of base liquidity to provide.
    /// - `quote_liquidity`: The amount of quote liquidity to provide.
    ///
    /// # Returns
    /// - `Ok(ProvidePayload)` containing the updated pool state and LP tokens to mint.
    /// - `Err(ErrorCode)` if any checks fail or calculations encounter errors.
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
            base_quote_ratio_sqrt: new_base_quote_ratio_sqrt,
            constant_product: new_constant_product_sqrt,
            base_liquidity: new_base_liquidity,
            quote_liquidity: new_quote_liquidity,
            lp_tokens_supply: new_lp_tokens_supply,
            lp_tokens_to_mint,
        })
    }

    /// Prepares the payload for withdrawing liquidity from the AMM.
    ///
    /// It calculates the amounts of base and quote liquidity to withdraw, ensuring the pool remains valid.
    ///
    /// # Parameters
    /// - `lp_tokens`: The number of LP tokens to redeem for liquidity withdrawal.
    ///
    /// # Returns
    /// - `Ok(WithdrawPayload)` containing the updated pool state and withdrawn liquidity amounts.
    /// - `Err(ErrorCode)` if any checks fail or calculations encounter errors.
    pub fn get_withdraw_payload(&self, lp_tokens: u64) -> Result<WithdrawPayload> {
        self.check_state()?;
        require!(lp_tokens > 0, ErrorCode::ProvidedLpTokensIsZero);

        let lp_tokens_left_supply = self.lp_tokens_supply.checked_sub(lp_tokens).ok_or(ErrorCode::WithdrawOverflowError)?;

        let (base_withdraw, quote_withdraw) = self.calculate_liquidity_from_share(lp_tokens).ok_or(ErrorCode::WithdrawLiquidityCalculationFailed)?;
        
        let new_base_liquidity = self.base_liquidity.checked_sub(base_withdraw).ok_or(ErrorCode::WithdrawOverflowError)?;
        let new_quote_liquidity = self.quote_liquidity.checked_sub(quote_withdraw).ok_or(ErrorCode::WithdrawOverflowError)?;

        // Checks that new base and quote liquidity don't equal zero and amm won't be drained
        let new_base_quote_ratio_sqrt = self.validate_and_calculate_liquidity_ratio(new_base_liquidity, new_quote_liquidity)?;

        Ok(WithdrawPayload{
            base_quote_ratio_sqrt: new_base_quote_ratio_sqrt,
            base_liquidity: new_base_liquidity,
            quote_liquidity: new_quote_liquidity,
            lp_tokens_supply: lp_tokens_left_supply,
            base_withdraw_amount: base_withdraw,
            quote_withdraw_amount: quote_withdraw,
        })
    }

    /// Computes the swap payload for exchanging tokens within the AMM.
    ///
    /// This function handles both **base-to-quote** and **quote-to-base** swaps.
    /// It calculates the updated pool state, applies provider and protocol fees,
    /// and validates the constant product invariant.
    ///
    /// # Parameters
    /// - `swap_amount`: The amount of tokens being swapped (either base or quote).
    /// - `estimated_result`: Expected amount of tokens to receive after the swap.
    /// - `allowed_slippage`: Maximum permissible deviation from `estimated_result`.
    /// - `providers_fee_rate_basis_points`: The liquidity provider's fee rate in basis points.
    /// - `protocol_fee_rate_basis_points`: The protocol fee rate in basis points.
    /// - `is_in_out`: `true` if swapping **base → quote**, `false` if swapping **quote → base**.
    ///
    /// # Returns
    /// - `Ok(SwapPayload)`: Contains the updated liquidity state and fees.
    /// - `Err(ErrorCode)`: If any validation fails (e.g., insufficient liquidity, overflow, or slippage exceeded).
    pub fn get_swap_payload(&self, swap_amount: u64, estimated_result: u64, allowed_slippage: u64, providers_fee_rate_basis_points: u16, protocol_fee_rate_basis_points: u16, is_in_out: bool) -> Result<SwapPayload> {
        self.check_state()?;
        require!(swap_amount > 0, ErrorCode::SwapAmountIsZero);
        require!(estimated_result > 0, ErrorCode::EstimatedResultIsZero);
        require!(providers_fee_rate_basis_points + protocol_fee_rate_basis_points <= 10000, ErrorCode::ConfigFeeRateExceeded);

        let (new_base_liquidity, new_quote_liquidity, amount_to_withdraw, protocol_fees_to_redeem);
        let providers_fees_to_redeem = Self::calculate_fee_amount(swap_amount, providers_fee_rate_basis_points);
        let protocol_fee_amount = Self::calculate_fee_amount(swap_amount, protocol_fee_rate_basis_points);
        if is_in_out {
            protocol_fees_to_redeem = self.protocol_base_fees_to_redeem.checked_add(protocol_fee_amount).ok_or(ErrorCode::SwapOverflowError)?;
            let base_amount_after_fees = swap_amount.checked_sub(providers_fees_to_redeem).unwrap().checked_sub(protocol_fee_amount).ok_or(ErrorCode::SwapOverflowError)?;
            (new_base_liquidity, new_quote_liquidity) = self.calculate_afterswap_liquidity(base_amount_after_fees, true).ok_or(ErrorCode::AfterswapCalculationFailed)?;
            amount_to_withdraw = self.quote_liquidity.checked_sub(new_quote_liquidity).ok_or(ErrorCode::SwapOverflowError)?;
        }
        else{
            protocol_fees_to_redeem = self.protocol_quote_fees_to_redeem.checked_add(protocol_fee_amount).ok_or(ErrorCode::SwapOverflowError)?;
            let quote_amount_after_fees = swap_amount.checked_sub(providers_fees_to_redeem).unwrap().checked_sub(protocol_fee_amount).ok_or(ErrorCode::SwapOverflowError)?;
            (new_base_liquidity, new_quote_liquidity) = self.calculate_afterswap_liquidity(quote_amount_after_fees, false).ok_or(ErrorCode::AfterswapCalculationFailed)?;
            amount_to_withdraw = self.base_liquidity.checked_sub(new_base_liquidity).ok_or(ErrorCode::SwapOverflowError)?;
        }
        
        // Check constant product change is in acceptable range
        self.validate_swap_constant_product(new_base_liquidity, new_quote_liquidity)?;
        Self::check_swap_result(amount_to_withdraw, estimated_result, allowed_slippage)?;
        
        Ok(SwapPayload::new(
            new_base_liquidity,
            new_quote_liquidity,
            protocol_fees_to_redeem,
            providers_fees_to_redeem,
            amount_to_withdraw,
            is_in_out,
        ))
    }

    /// Prepares the payload for collecting protocol fees from the AMM.
    ///
    /// This method checks if there are any protocol fees available for redemption and creates
    /// a `CollectFeesPayload` containing the amounts of base and quote token fees.
    ///
    /// # Returns
    /// - `Ok(CollectFeesPayload)`: Contains the protocol fees available for redemption for both base and quote tokens.
    /// - `Err(ErrorCode::ProvidersFeesIsZero)`: If both `protocol_base_fees_to_redeem` and `protocol_quote_fees_to_redeem` are zero, meaning no fees are available to collect.
    #[inline]
    pub fn get_collect_fees_payload(&self) -> Result<CollectFeesPayload>{
        require!(self.protocol_base_fees_to_redeem > 0 || self.protocol_quote_fees_to_redeem > 0, ErrorCode::ProvidersFeesIsZero);
        Ok(CollectFeesPayload::new(
            self.protocol_base_fees_to_redeem,
            self.protocol_quote_fees_to_redeem,
            0,
            0
        ))
    }
}

impl CpAmm {

    /// Initializes the AMM with the provided token mints and configuration.
    ///
    /// This method sets the initial configuration for the AMM, linking it with
    /// the provided base, quote, and LP token mints, and a configuration account.
    /// It also initializes the protocol and provider fee rates and marks the AMM as initialized.
    ///
    /// # Parameters
    /// - `base_mint`: The mint of the base token.
    /// - `quote_mint`: The mint of the quote token.
    /// - `lp_mint`: The mint of the LP token.
    /// - `amms_config`: The configuration account for the AMM.
    /// - `bump`: The canonical bump seed for the AMM's PDA.
    ///
    /// # Returns
    /// - `Ok(())` if the initialization is successful.
    /// - `Err(ErrorCode)` if the AMM is already initialized.
    pub fn initialize(
        &mut self,
        base_mint: &InterfaceAccount<token_interface::Mint>,
        quote_mint: &InterfaceAccount<token_interface::Mint>,
        lp_mint: &Account<Mint>,
        amms_config: &Account<AmmsConfig>,
        bump: u8,
    ) -> Result<()>{
        require!(!self.is_initialized, ErrorCode::CpAmmAlreadyInitialized);

        self.base_mint = base_mint.key();
        self.quote_mint = quote_mint.key();
        self.lp_mint = lp_mint.key();
        self.amms_config = amms_config.key();
        self.is_launched = false;
        self.is_initialized = true;
        self.bump = [bump];

        Ok(())
    }

    /// Launches the AMM with the provided liquidity and vaults.
    ///
    /// This method finalizes the initial setup of the AMM by locking in the provided
    /// base and quote liquidity, initializing the constant product and liquidity ratios,
    /// and linking the vault accounts.
    ///
    /// # Parameters
    /// - `launch_payload`: Contains the initial liquidity, LP token supply, and ratios.
    /// - `base_vault`: The vault holding the base tokens.
    /// - `quote_vault`: The vault holding the quote tokens.
    /// - `locked_lp_vault`: The vault holding locked LP tokens.
    ///
    /// # Returns
    /// - No return value. Modifies the internal state of the AMM.
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

    /// Updates the AMM state after liquidity is provided.
    ///
    /// This method updates the base and quote liquidity, LP token supply,
    /// constant product, and liquidity ratio based on the provided payload.
    ///
    /// # Parameters
    /// - `provide_payload`: Contains the new liquidity amounts, LP tokens to mint, and ratios.
    ///
    /// # Returns
    /// - No return value. Modifies the internal state of the AMM.
    pub(crate) fn provide(&mut self, provide_payload: ProvidePayload) -> (){
        self.base_liquidity = provide_payload.base_liquidity;
        self.quote_liquidity = provide_payload.quote_liquidity;
        self.lp_tokens_supply = provide_payload.lp_tokens_supply;
        self.constant_product_sqrt = provide_payload.constant_product;
        self.base_quote_ratio_sqrt = provide_payload.base_quote_ratio_sqrt;
    }

    /// Updates the AMM state after liquidity is withdrawn.
    ///
    /// This method updates the base and quote liquidity, LP token supply,
    /// constant product, and liquidity ratio after liquidity is redeemed from the pool.
    ///
    /// # Parameters
    /// - `withdraw_payload`: Contains the new liquidity amounts and LP tokens to burn.
    ///
    /// # Returns
    /// - No return value. Modifies the internal state of the AMM.
    pub(crate) fn withdraw(&mut self, withdraw_payload: WithdrawPayload) -> (){
        self.base_liquidity = withdraw_payload.base_liquidity;
        self.quote_liquidity = withdraw_payload.quote_liquidity;
        self.lp_tokens_supply = withdraw_payload.lp_tokens_supply;
        self.constant_product_sqrt = Self::calculate_constant_product_sqrt(self.base_liquidity, self.quote_liquidity).unwrap();
        self.base_quote_ratio_sqrt = withdraw_payload.base_quote_ratio_sqrt;
    }

    /// Updates the AMM state after a token swap operation.
    ///
    /// This method adjusts the base and quote liquidity, protocol fees, providers fees, constant product,
    /// and liquidity ratio after a swap. It ensures the AMM remains consistent with the
    /// constant product invariant.
    ///
    /// # Parameters
    /// - `swap_payload`: Contains the updated liquidity values, fees, and swap details.
    ///
    /// # Returns
    /// - No return value. Modifies the internal state of the AMM.
    pub(crate) fn swap(&mut self, swap_payload: SwapPayload) {
        self.base_liquidity = swap_payload.base_liquidity;
        self.quote_liquidity = swap_payload.quote_liquidity;
        if swap_payload.is_in_out{
            self.protocol_base_fees_to_redeem = swap_payload.protocol_fees_to_redeem;
            self.base_liquidity += swap_payload.providers_fees_to_redeem
        }
        else{
            self.protocol_quote_fees_to_redeem = swap_payload.protocol_fees_to_redeem;
            self.quote_liquidity += swap_payload.providers_fees_to_redeem
        }
        self.constant_product_sqrt = Self::calculate_constant_product_sqrt(self.base_liquidity, self.quote_liquidity).unwrap();
        self.base_quote_ratio_sqrt = Self::calculate_base_quote_ratio_sqrt(self.base_liquidity, self.quote_liquidity).unwrap();
    }

    /// Updates the protocol fees for the AMM based on the provided payload.
    ///
    /// This method sets the protocol fees available for redemption to the updated values
    /// specified in the `CollectFeesPayload`.
    ///
    /// # Parameters
    /// - `collect_fees_payload`: A `CollectFeesPayload` containing the updated protocol fees
    ///   for both base and quote tokens.
    ///
    /// # Returns
    /// - None. This method directly modifies the internal state of the AMM.
    pub(crate) fn collect_fees(&mut self, collect_fees_payload: CollectFeesPayload) {
        self.protocol_base_fees_to_redeem = collect_fees_payload.new_protocol_base_fees_to_redeem;
        self.protocol_quote_fees_to_redeem = collect_fees_payload.new_protocol_quote_fees_to_redeem;
    }

}

#[cfg(test)]
mod cp_amm_tests {
    use anchor_lang::Discriminator;
    use crate::constants::ANCHOR_DISCRIMINATOR;
    use super::*;

    #[derive(Default)]
    struct CpAmmBuilder {
        is_initialized: bool,
        is_launched: bool,
        initial_locked_liquidity: u64,
        constant_product_sqrt: Q64_128,
        base_quote_ratio_sqrt: Q64_128,
        base_liquidity: u64,
        quote_liquidity: u64,
        lp_tokens_supply: u64,
        protocol_base_fees_to_redeem: u64,
        protocol_quote_fees_to_redeem: u64,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        lp_mint: Pubkey,
        base_vault: Pubkey,
        quote_vault: Pubkey,
        locked_lp_vault: Pubkey,
        amms_config: Pubkey,
        bump: [u8; 1],
    }

    impl CpAmmBuilder {
        fn new() -> Self {
            Self{
                ..Default::default()
            }
        }

        fn is_initialized(mut self, value: bool) -> Self {
            self.is_initialized = value;
            self
        }

        fn is_launched(mut self, value: bool) -> Self {
            self.is_launched = value;
            self
        }

        fn initial_locked_liquidity(mut self, value: u64) -> Self {
            self.initial_locked_liquidity = value;
            self
        }

        fn constant_product_sqrt(mut self, value: Q64_128) -> Self {
            self.constant_product_sqrt = value;
            self
        }

        fn base_quote_ratio_sqrt(mut self, value: Q64_128) -> Self {
            self.base_quote_ratio_sqrt = value;
            self
        }

        fn base_liquidity(mut self, value: u64) -> Self {
            self.base_liquidity = value;
            self
        }

        fn quote_liquidity(mut self, value: u64) -> Self {
            self.quote_liquidity = value;
            self
        }

        fn lp_tokens_supply(mut self, value: u64) -> Self {
            self.lp_tokens_supply = value;
            self
        }

        fn protocol_base_fees_to_redeem(mut self, value: u64) -> Self {
            self.protocol_base_fees_to_redeem = value;
            self
        }

        fn protocol_quote_fees_to_redeem(mut self, value: u64) -> Self {
            self.protocol_quote_fees_to_redeem = value;
            self
        }

        fn base_mint(mut self, value: Pubkey) -> Self {
            self.base_mint = value;
            self
        }

        fn quote_mint(mut self, value: Pubkey) -> Self {
            self.quote_mint = value;
            self
        }

        fn lp_mint(mut self, value: Pubkey) -> Self {
            self.lp_mint = value;
            self
        }

        fn base_vault(mut self, value: Pubkey) -> Self {
            self.base_vault = value;
            self
        }

        fn quote_vault(mut self, value: Pubkey) -> Self {
            self.quote_vault = value;
            self
        }

        fn locked_lp_vault(mut self, value: Pubkey) -> Self {
            self.locked_lp_vault = value;
            self
        }

        fn amms_config(mut self, value: Pubkey) -> Self {
            self.amms_config = value;
            self
        }

        fn bump(mut self, value: [u8; 1]) -> Self {
            self.bump = value;
            self
        }

        fn build(self) -> CpAmm {
            CpAmm {
                is_initialized: self.is_initialized,
                is_launched: self.is_launched,
                initial_locked_liquidity: self.initial_locked_liquidity,
                constant_product_sqrt: self.constant_product_sqrt,
                base_quote_ratio_sqrt: self.base_quote_ratio_sqrt,
                base_liquidity: self.base_liquidity,
                quote_liquidity: self.quote_liquidity,
                lp_tokens_supply: self.lp_tokens_supply,
                protocol_base_fees_to_redeem: self.protocol_base_fees_to_redeem,
                protocol_quote_fees_to_redeem: self.protocol_quote_fees_to_redeem,
                base_mint: self.base_mint,
                quote_mint: self.quote_mint,
                lp_mint: self.lp_mint,
                base_vault: self.base_vault,
                quote_vault: self.quote_vault,
                locked_lp_vault: self.locked_lp_vault,
                amms_config: self.amms_config,
                bump: self.bump,
            }
        }
    }

    /// Tests `CpAmm` account data layout.
    #[test]
    fn test_cp_amm_data_layout(){
        let is_initialized = true;
        let is_launched = true;
        let initial_locked_liquidity = 1_000_000u64;
        let constant_product_sqrt = Q64_128::from_u64(2_000_000);
        let base_quote_ratio_sqrt = Q64_128::from_u64(1_000_000);
        let base_liquidity = 500_000u64;
        let quote_liquidity = 250_000u64;
        let lp_tokens_supply = 100_000u64;
        let protocol_base_fees_to_redeem = 1_000u64;
        let protocol_quote_fees_to_redeem = 500u64;
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();
        let lp_mint = Pubkey::new_unique();
        let base_vault = Pubkey::new_unique();
        let quote_vault = Pubkey::new_unique();
        let locked_lp_vault = Pubkey::new_unique();
        let amms_config = Pubkey::new_unique();
        let bump = [42u8];
        
        let mut data = [0u8; ANCHOR_DISCRIMINATOR + 323];
        let mut offset = 0;

        data[offset..offset + ANCHOR_DISCRIMINATOR].copy_from_slice(&CpAmm::discriminator()); offset += ANCHOR_DISCRIMINATOR;
        data[offset] = is_initialized as u8; offset += 1;
        data[offset] = is_launched as u8; offset += 1;
        data[offset..offset + 8].copy_from_slice(&initial_locked_liquidity.to_le_bytes()); offset += 8;
        data[offset..offset + 16].copy_from_slice(&constant_product_sqrt.get_fractional_bits().to_le_bytes()); offset += 16;
        data[offset..offset + 8].copy_from_slice(&constant_product_sqrt.get_integer_bits().to_le_bytes()); offset += 8;
        data[offset..offset + 16].copy_from_slice(&base_quote_ratio_sqrt.get_fractional_bits().to_le_bytes()); offset += 16;
        data[offset..offset + 8].copy_from_slice(&base_quote_ratio_sqrt.get_integer_bits().to_le_bytes()); offset += 8;
        data[offset..offset + 8].copy_from_slice(&base_liquidity.to_le_bytes()); offset += 8;
        data[offset..offset + 8].copy_from_slice(&quote_liquidity.to_le_bytes()); offset += 8;
        data[offset..offset + 8].copy_from_slice(&lp_tokens_supply.to_le_bytes()); offset += 8;
        data[offset..offset + 8].copy_from_slice(&protocol_base_fees_to_redeem.to_le_bytes()); offset += 8;
        data[offset..offset + 8].copy_from_slice(&protocol_quote_fees_to_redeem.to_le_bytes()); offset += 8;
        data[offset..offset + 32].copy_from_slice(base_mint.as_ref()); offset += 32;
        data[offset..offset + 32].copy_from_slice(quote_mint.as_ref()); offset += 32;
        data[offset..offset + 32].copy_from_slice(lp_mint.as_ref()); offset += 32;
        data[offset..offset + 32].copy_from_slice(base_vault.as_ref()); offset += 32;
        data[offset..offset + 32].copy_from_slice(quote_vault.as_ref()); offset += 32;
        data[offset..offset + 32].copy_from_slice(locked_lp_vault.as_ref()); offset += 32;
        data[offset..offset + 32].copy_from_slice(amms_config.as_ref()); offset += 32;
        data[offset] = bump[0]; offset += 1;
        
        assert_eq!(ANCHOR_DISCRIMINATOR + CpAmm::INIT_SPACE, offset);

        let deserialized_cp_amm = CpAmm::try_deserialize(&mut data.as_ref()).unwrap();

        assert_eq!(deserialized_cp_amm.is_initialized, is_initialized);
        assert_eq!(deserialized_cp_amm.is_launched, is_launched);
        assert_eq!(deserialized_cp_amm.initial_locked_liquidity, initial_locked_liquidity);
        assert_eq!(deserialized_cp_amm.constant_product_sqrt, constant_product_sqrt);
        assert_eq!(deserialized_cp_amm.base_quote_ratio_sqrt, base_quote_ratio_sqrt);
        assert_eq!(deserialized_cp_amm.base_liquidity, base_liquidity);
        assert_eq!(deserialized_cp_amm.quote_liquidity, quote_liquidity);
        assert_eq!(deserialized_cp_amm.lp_tokens_supply, lp_tokens_supply);
        assert_eq!(deserialized_cp_amm.protocol_base_fees_to_redeem, protocol_base_fees_to_redeem);
        assert_eq!(deserialized_cp_amm.protocol_quote_fees_to_redeem, protocol_quote_fees_to_redeem);
        assert_eq!(deserialized_cp_amm.base_mint, base_mint);
        assert_eq!(deserialized_cp_amm.quote_mint, quote_mint);
        assert_eq!(deserialized_cp_amm.lp_mint, lp_mint);
        assert_eq!(deserialized_cp_amm.base_vault, base_vault);
        assert_eq!(deserialized_cp_amm.quote_vault, quote_vault);
        assert_eq!(deserialized_cp_amm.locked_lp_vault, locked_lp_vault);
        assert_eq!(deserialized_cp_amm.amms_config, amms_config);
        assert_eq!(deserialized_cp_amm.bump, bump);
        
        let mut serialized_cp_amm = Vec::new();
        deserialized_cp_amm.try_serialize(&mut serialized_cp_amm).unwrap();
        assert_eq!(serialized_cp_amm.as_slice(), data.as_ref());
    }
    
    /// Tests getter methods of the `CpAmm` struct.
    #[test]
    fn test_cp_amm_getters() {
        let default_pubkey = Pubkey::new_unique();

        let amm = CpAmmBuilder::new()
            .is_initialized(true)
            .is_launched(false)
            .initial_locked_liquidity(1000)
            .constant_product_sqrt(Q64_128::from_u64(2000))
            .base_quote_ratio_sqrt(Q64_128::from_u64(3000))
            .base_liquidity(4000)
            .quote_liquidity(5000)
            .lp_tokens_supply(6000)
            .base_mint(default_pubkey)
            .quote_mint(default_pubkey)
            .lp_mint(default_pubkey)
            .base_vault(default_pubkey)
            .quote_vault(default_pubkey)
            .locked_lp_vault(default_pubkey)
            .amms_config(default_pubkey)
            .bump([253])
            .build();

        assert!(amm.is_initialized());
        assert!(!amm.is_launched());
        assert_eq!(amm.bump(), 253);
        assert_eq!(amm.base_mint(), &default_pubkey);
        assert_eq!(amm.quote_mint(), &default_pubkey);
        assert_eq!(amm.lp_mint, default_pubkey);
        assert_eq!(amm.base_vault(), &default_pubkey);
        assert_eq!(amm.quote_vault(), &default_pubkey);
        assert_eq!(amm.amms_config(), &default_pubkey);

        assert_eq!(amm.constant_product_sqrt(), Q64_128::from_u64(2000));
        assert_eq!(amm.base_quote_ratio_sqrt(), Q64_128::from_u64(3000));
        assert_eq!(amm.base_liquidity(), 4000);
        assert_eq!(amm.quote_liquidity(), 5000);
        assert_eq!(amm.lp_tokens_supply(), 6000);
    }
    
    mod state_change_tests {
        use super::*;

        /// Tests the `provide` method of `CpAmm`.
        #[test]
        fn test_provide() {
            let mut amm = CpAmmBuilder::new().build();

            let provide_payload = ProvidePayload::new(
                Q64_128::from_u64(2),
                Q64_128::from_u64(2000),
                4000,
                1000,
                6000,
                1000,
            );

            amm.provide(provide_payload);

            assert_eq!(amm.base_liquidity, 4000);
            assert_eq!(amm.quote_liquidity, 1000);
            assert_eq!(amm.lp_tokens_supply, 6000);
            assert_eq!(amm.base_quote_ratio_sqrt, Q64_128::from_u64(2));
            assert_eq!(amm.constant_product_sqrt, Q64_128::from_u64(2000));
        }

        /// Tests the `withdraw` method of `CpAmm`.
        #[test]
        fn test_withdraw() {
            let mut amm = CpAmmBuilder::new().build();

            let withdraw_payload = WithdrawPayload::new(
                Q64_128::from_u64(2),
                4000,
                1000,
                5000,
                400,
                100,
            );

            amm.withdraw(withdraw_payload);

            assert_eq!(amm.base_liquidity, 4000);
            assert_eq!(amm.quote_liquidity, 1000);
            assert_eq!(amm.lp_tokens_supply, 5000);
            assert_eq!(amm.base_quote_ratio_sqrt, Q64_128::from_u64(2));
            assert_eq!(amm.constant_product_sqrt, Q64_128::from_u64(2000));
        }

        /// Tests the `swap` method of `CpAmm`.
        #[test]
        fn test_swap() {
            let mut amm = CpAmmBuilder::new().build();

            let swap_payload_in = SwapPayload::new(3980, 1000, 1, 20, 100, true);
            let swap_payload_out = SwapPayload::new(1000, 985, 15, 15, 100, false);

            amm.swap(swap_payload_in);
            assert_eq!(amm.base_liquidity, 4000);
            assert_eq!(amm.quote_liquidity, 1000);
            assert_eq!(amm.protocol_base_fees_to_redeem, 1);
            assert_eq!(amm.constant_product_sqrt, Q64_128::from_u64(2000));
            assert_eq!(amm.base_quote_ratio_sqrt, Q64_128::from_u64(2));

            amm.swap(swap_payload_out);
            assert_eq!(amm.base_liquidity, 1000);
            assert_eq!(amm.quote_liquidity, 1000);
            assert_eq!(amm.protocol_base_fees_to_redeem, 1);
            assert_eq!(amm.protocol_quote_fees_to_redeem, 15);
            assert_eq!(amm.constant_product_sqrt, Q64_128::from_u64(1000));
            assert_eq!(amm.base_quote_ratio_sqrt, Q64_128::from_u64(1));
        }

        /// Tests the `collect_fees` method of `CpAmm`.
        #[test]
        fn test_collect_fees() {
            let mut amm = CpAmmBuilder::new().protocol_base_fees_to_redeem(123213).protocol_quote_fees_to_redeem(213442).build();

            let collect_fees_payload = CollectFeesPayload::new(123213, 213442, 0,0);

            amm.collect_fees(collect_fees_payload);
            assert_eq!(amm.protocol_base_fees_to_redeem, 0);
            assert_eq!(amm.protocol_quote_fees_to_redeem, 0);
        }
    }
    
    mod operations_calculations_tests {
        use super::*;

        /// Tests the `check_state` method of `CpAmm`.
        #[test]
        fn test_check_state() {
            let amm1 = CpAmmBuilder::new()
                .is_launched(false)
                .base_liquidity(1000)
                .quote_liquidity(1000)
                .lp_tokens_supply(0)
                .build();
            
            let amm2 = CpAmmBuilder::new()
                .is_launched(true)
                .base_liquidity(1000)
                .quote_liquidity(1000)
                .lp_tokens_supply(5000)
                .build();
            assert!(amm1.check_state().is_err());
            assert!(amm2.check_state().is_ok());
        }

        /// Tests the `get_launch_payload` method of `CpAmm`.
        #[test]
        fn test_get_launch_payload() {
            let amm = CpAmmBuilder::new()
                .is_initialized(true)
                .is_launched(false)
                .build();

            let base_liquidity = 400000;
            let quote_liquidity = 400000;

            let payload = amm.get_launch_payload(base_liquidity, quote_liquidity).unwrap();

            assert_eq!(payload.base_liquidity, 400000);
            assert_eq!(payload.quote_liquidity, 400000);
            assert_eq!(payload.constant_product_sqrt, Q64_128::from_u64(400000));
            assert_eq!(payload.base_quote_ratio_sqrt, Q64_128::from_u64(1));
            assert_eq!(payload.lp_tokens_supply, payload.constant_product_sqrt.as_u64());
            assert_eq!(payload.initial_locked_liquidity, CpAmm::INITIAL_LOCKED_LP_TOKENS);
            
            assert!(amm.get_launch_payload(5500, 1000).is_err());
        }

        /// Tests the `get_provide_payload` method of `CpAmm`.
        #[test]
        fn test_get_provide_payload() {
            let initial_base_liquidity = 4_000_000;
            let initial_quote_liquidity = 1_000_000;
            let initial_constant_product_sqrt = Q64_128::from_u64(2_000_000);
            let initial_base_quote_ratio_sqrt = Q64_128::from_u64(2);
            let initial_lp_tokens_supply = 2_000_000;

            let amm = CpAmmBuilder::new()
                .is_launched(true)
                .base_liquidity(initial_base_liquidity)
                .quote_liquidity(initial_quote_liquidity)
                .constant_product_sqrt(initial_constant_product_sqrt)
                .base_quote_ratio_sqrt(initial_base_quote_ratio_sqrt)
                .lp_tokens_supply(initial_lp_tokens_supply)
                .build();

            let provided_base_liquidity = 2_000_000;
            let provided_quote_liquidity = 500_000;

            let payload = amm.get_provide_payload(provided_base_liquidity, provided_quote_liquidity).unwrap();

            let expected_base_liquidity = initial_base_liquidity + provided_base_liquidity;
            let expected_quote_liquidity = initial_quote_liquidity + provided_quote_liquidity;
            let expected_constant_product_sqrt = Q64_128::from_u64(3_000_000);
            let expected_lp_tokens_to_mint = 1_000_000;
            let expected_lp_tokens_supply = initial_lp_tokens_supply + expected_lp_tokens_to_mint;
            
            assert_eq!(payload.base_liquidity, expected_base_liquidity);
            assert_eq!(payload.quote_liquidity, expected_quote_liquidity);
            assert_eq!(payload.base_quote_ratio_sqrt, initial_base_quote_ratio_sqrt);
            assert_eq!(payload.constant_product, expected_constant_product_sqrt);
            assert_eq!(payload.lp_tokens_to_mint, expected_lp_tokens_to_mint);
            assert_eq!(payload.lp_tokens_supply, expected_lp_tokens_supply);
        }

        /// Tests the `get_withdraw_payload` method of `CpAmm`.
        #[test]
        fn test_get_withdraw_payload() {
            let initial_base_liquidity = 6_000_000;
            let initial_quote_liquidity = 1_500_000;
            let initial_constant_product_sqrt = Q64_128::from_u64(3_000_000);
            let initial_base_quote_ratio_sqrt = Q64_128::from_u64(2);
            let initial_lp_tokens_supply = 3_000_000;

            let amm = CpAmmBuilder::new()
                .is_launched(true)
                .base_liquidity(initial_base_liquidity)
                .quote_liquidity(initial_quote_liquidity)
                .constant_product_sqrt(initial_constant_product_sqrt)
                .base_quote_ratio_sqrt(initial_base_quote_ratio_sqrt)
                .lp_tokens_supply(initial_lp_tokens_supply)
                .build();

            let lp_tokens_withdraw = 1000000;

            let payload = amm.get_withdraw_payload(lp_tokens_withdraw).unwrap();

            let expected_base_withdraw_amount = 2_000_000;
            let expected_quote_withdraw_amount = 500_000;
            let expected_base_liquidity = initial_base_liquidity - expected_base_withdraw_amount;
            let expected_quote_liquidity = initial_quote_liquidity - expected_quote_withdraw_amount;
            let expected_lp_tokens_supply = initial_lp_tokens_supply - lp_tokens_withdraw;

            assert_eq!(payload.base_liquidity, expected_base_liquidity);
            assert_eq!(payload.quote_liquidity, expected_quote_liquidity);
            assert_eq!(payload.base_quote_ratio_sqrt, initial_base_quote_ratio_sqrt);
            assert_eq!(payload.base_withdraw_amount, expected_base_withdraw_amount);
            assert_eq!(payload.quote_withdraw_amount, expected_quote_withdraw_amount);
            assert_eq!(payload.lp_tokens_supply, expected_lp_tokens_supply);
        }

        /// Tests the `get_swap_payload` method of `CpAmm` for in->out swap.
        #[test]
        fn test_get_base_to_quote_swap_payload() {
            let initial_base_liquidity = 6_000_000;
            let initial_quote_liquidity = 1_500_000;
            let protocol_fee_basis_points = 100;
            let providers_fee_basis_points = 100;
            let initial_constant_product_sqrt = Q64_128::from_u64(3_000_000);
            let initial_base_quote_ratio_sqrt = Q64_128::from_u64(2);
            let initial_lp_tokens_supply = 3_000_000;
                
            let amm = CpAmmBuilder::new()
                .is_launched(true)
                .base_liquidity(initial_base_liquidity)
                .quote_liquidity(initial_quote_liquidity)
                .constant_product_sqrt(initial_constant_product_sqrt)
                .base_quote_ratio_sqrt(initial_base_quote_ratio_sqrt)
                .lp_tokens_supply(initial_lp_tokens_supply)
                .build();
        
            let base_amount: u64 = 3_061_224;
            let protocol_fee = base_amount * protocol_fee_basis_points as u64 / 10000;
            let providers_fee = base_amount * providers_fee_basis_points as u64 / 10000;
            let estimated_result = 500_000;
            let allowed_slippage = 0;

            
            let payload = amm.get_swap_payload(base_amount, estimated_result, allowed_slippage, providers_fee_basis_points, protocol_fee_basis_points, true).unwrap();
        
            assert_eq!(payload.base_liquidity, initial_base_liquidity + base_amount - protocol_fee - providers_fee);
            assert_eq!(payload.quote_liquidity, initial_quote_liquidity - estimated_result);
            assert_eq!(payload.protocol_fees_to_redeem, protocol_fee);
            assert_eq!(payload.providers_fees_to_redeem, providers_fee);
            assert_eq!(payload.amount_to_withdraw, estimated_result);
            assert!(payload.is_in_out);
        }

        /// Tests the `get_swap_payload` method of `CpAmm` for out->in swap.
        #[test]
        fn test_get_quote_to_base_swap_payload() {
            let initial_base_liquidity = 6_000_000;
            let initial_quote_liquidity = 1_500_000;
            let protocol_fee_basis_points = 100;
            let providers_fee_basis_points = 100;
            let initial_constant_product_sqrt = Q64_128::from_u64(3_000_000);
            let initial_base_quote_ratio_sqrt = Q64_128::from_u64(2);
            let initial_lp_tokens_supply = 3_000_000;
                
            let amm = CpAmmBuilder::new()
                .is_launched(true)
                .base_liquidity(initial_base_liquidity)
                .quote_liquidity(initial_quote_liquidity)
                .constant_product_sqrt(initial_constant_product_sqrt)
                .base_quote_ratio_sqrt(initial_base_quote_ratio_sqrt)
                .lp_tokens_supply(initial_lp_tokens_supply)
                .build();

            let quote_amount: u64 = 510_204;
            let protocol_fee = quote_amount * protocol_fee_basis_points as u64 / 10000;
            let providers_fee = quote_amount * providers_fee_basis_points as u64 / 10000;
            let estimated_result = 1_500_000;
            let allowed_slippage = 0;

            let payload = amm.get_swap_payload(quote_amount, estimated_result, allowed_slippage, providers_fee_basis_points, protocol_fee_basis_points, false).unwrap();

            assert_eq!(payload.base_liquidity, initial_base_liquidity - estimated_result);
            assert_eq!(payload.quote_liquidity, initial_quote_liquidity + quote_amount - protocol_fee - providers_fee);
            assert_eq!(payload.protocol_fees_to_redeem, protocol_fee);
            assert_eq!(payload.providers_fees_to_redeem, providers_fee);
            assert_eq!(payload.amount_to_withdraw, estimated_result);
            assert!(!payload.is_in_out);
        }

        /// Tests the `get_collect_fees_payload` method of `CpAmm`.
        #[test]
        fn test_get_collect_fees_payload() {
            let protocol_base_fees_to_redeem = 1234353;
            let protocol_quote_fees_to_redeem = 67574567;
            let amm = CpAmmBuilder::new()
                .protocol_base_fees_to_redeem(protocol_base_fees_to_redeem)
                .protocol_quote_fees_to_redeem(protocol_quote_fees_to_redeem)
                .build();

            let payload = amm.get_collect_fees_payload().unwrap();

            assert_eq!(payload.protocol_base_fees_to_redeem, protocol_base_fees_to_redeem);
            assert_eq!(payload.protocol_quote_fees_to_redeem, protocol_quote_fees_to_redeem);
            assert_eq!(payload.new_protocol_base_fees_to_redeem, 0);
            assert_eq!(payload.new_protocol_quote_fees_to_redeem, 0);
        }
    }
}

/// Represents the data required to launch the AMM.
///
/// This struct contains the initial parameters for the pool, including
/// the locked liquidity, initial ratios, and the supply of LP tokens.
///
/// # Fields
/// - `initial_locked_liquidity`: The amount of liquidity that will be locked in the pool upon launch.
/// - `constant_product_sqrt`: The square root of the constant product invariant.
/// - `base_quote_ratio_sqrt`: The square root of the ratio between base and quote liquidity.
/// - `base_liquidity`: The initial base token liquidity in the pool.
/// - `quote_liquidity`: The initial quote token liquidity in the pool.
/// - `lp_tokens_supply`: The total supply of LP tokens minted upon launch.
#[derive(Debug)]
pub struct LaunchPayload {
    initial_locked_liquidity: u64,
    constant_product_sqrt: Q64_128,
    base_quote_ratio_sqrt: Q64_128,
    base_liquidity: u64,
    quote_liquidity: u64,
    lp_tokens_supply: u64,
}
impl LaunchPayload {
    /// Creates a new `LaunchPayload` instance with the specified parameters.
    ///
    /// # Parameters
    /// - `initial_locked_liquidity`: The locked liquidity amount.
    /// - `constant_product_sqrt`: The square root of the constant product.
    /// - `base_quote_ratio_sqrt`: The square root of the base-to-quote liquidity ratio.
    /// - `base_liquidity`: The base token liquidity.
    /// - `quote_liquidity`: The quote token liquidity.
    /// - `lp_tokens_supply`: The total LP token supply.
    pub fn new(
        initial_locked_liquidity: u64,
        constant_product_sqrt: Q64_128,
        base_quote_ratio_sqrt: Q64_128,
        base_liquidity: u64,
        quote_liquidity: u64,
        lp_tokens_supply: u64,
    ) -> Self {
        Self {
            initial_locked_liquidity,
            constant_product_sqrt,
            base_quote_ratio_sqrt,
            base_liquidity,
            quote_liquidity,
            lp_tokens_supply,
        }
    }

    /// Returns the amount of initially locked liquidity.
    pub fn initial_locked_liquidity(&self) -> u64{
        self.initial_locked_liquidity
    }

    /// Returns the total liquidity available for use after subtracting the locked liquidity.
    pub fn launch_liquidity(&self) -> u64{
        self.lp_tokens_supply.checked_sub(self.initial_locked_liquidity).unwrap()
    }
}

/// Represents the data required to provide liquidity to the AMM.
///
/// This struct contains the updated state of the pool after liquidity is added,
/// including the adjusted ratios, constant product, and LP tokens to mint.
///
/// # Fields
/// - `base_quote_ratio_sqrt`: The updated square root of the base-to-quote liquidity ratio.
/// - `constant_product`: The updated square root of the constant product.
/// - `base_liquidity`: The updated base token liquidity in the pool.
/// - `quote_liquidity`: The updated quote token liquidity in the pool.
/// - `lp_tokens_supply`: The updated total supply of LP tokens.
/// - `lp_tokens_to_mint`: The number of LP tokens to mint for the liquidity provider.
#[derive(Debug)]
pub struct ProvidePayload {
    base_quote_ratio_sqrt: Q64_128,
    constant_product: Q64_128,
    base_liquidity: u64,
    quote_liquidity: u64,
    lp_tokens_supply: u64,
    lp_tokens_to_mint: u64,
}
impl ProvidePayload {
    /// Creates a new `ProvidePayload` instance with the specified parameters.
    ///
    /// # Parameters
    /// - `base_quote_ratio_sqrt`: The updated square root of the liquidity ratio.
    /// - `constant_product`: The updated square root of the constant product.
    /// - `base_liquidity`: The updated base liquidity amount.
    /// - `quote_liquidity`: The updated quote liquidity amount.
    /// - `lp_tokens_supply`: The updated LP token supply.
    /// - `lp_tokens_to_mint`: The LP tokens to mint for the provider.
    pub fn new(
        base_quote_ratio_sqrt: Q64_128,
        constant_product: Q64_128,
        base_liquidity: u64,
        quote_liquidity: u64,
        lp_tokens_supply: u64,
        lp_tokens_to_mint: u64,
    ) -> Self {
        Self {
            base_quote_ratio_sqrt,
            constant_product,
            base_liquidity,
            quote_liquidity,
            lp_tokens_supply,
            lp_tokens_to_mint,
        }
    }
    
    /// Returns the number of LP tokens to mint for the liquidity provider.
    pub fn lp_tokens_to_mint(&self) -> u64{
        self.lp_tokens_to_mint
    }
}

/// Represents the data required to withdraw liquidity from the AMM.
///
/// This struct contains the updated state of the pool and the amounts
/// of base and quote tokens withdrawn after liquidity is removed.
///
/// # Fields
/// - `base_quote_ratio_sqrt`: The updated square root of the base-to-quote liquidity ratio.
/// - `base_liquidity`: The updated base token liquidity in the pool.
/// - `quote_liquidity`: The updated quote token liquidity in the pool.
/// - `lp_tokens_supply`: The updated total supply of LP tokens.
/// - `base_withdraw_amount`: The amount of base tokens withdrawn.
/// - `quote_withdraw_amount`: The amount of quote tokens withdrawn.
#[derive(Debug)]
pub struct WithdrawPayload{
    base_quote_ratio_sqrt: Q64_128,
    base_liquidity: u64,
    quote_liquidity: u64,
    lp_tokens_supply: u64,
    base_withdraw_amount: u64,
    quote_withdraw_amount: u64
}
impl WithdrawPayload {
    /// Creates a new `WithdrawPayload` instance with the specified parameters.
    ///
    /// # Parameters
    /// - `base_quote_ratio_sqrt`: The updated square root of the liquidity ratio.
    /// - `base_liquidity`: The updated base liquidity amount.
    /// - `quote_liquidity`: The updated quote liquidity amount.
    /// - `lp_tokens_supply`: The updated LP token supply.
    /// - `base_withdraw_amount`: The base tokens withdrawn.
    /// - `quote_withdraw_amount`: The quote tokens withdrawn.
    pub fn new(
        base_quote_ratio_sqrt: Q64_128,
        base_liquidity: u64,
        quote_liquidity: u64,
        lp_tokens_supply: u64,
        base_withdraw_amount: u64,
        quote_withdraw_amount: u64,
    ) -> Self {
        Self {
            base_quote_ratio_sqrt,
            base_liquidity,
            quote_liquidity,
            lp_tokens_supply,
            base_withdraw_amount,
            quote_withdraw_amount,
        }
    }

    /// Returns the amount of base tokens withdrawn.
    pub fn base_withdraw_amount(&self) -> u64{
        self.base_withdraw_amount
    }


    /// Returns the amount of quote tokens withdrawn.
    pub fn quote_withdraw_amount(&self) -> u64{
        self.quote_withdraw_amount
    }
}

/// Represents the data required for a token swap operation in the AMM.
///
/// This struct contains the updated state of the pool after a swap
/// and the fees generated during the operation.
///
/// # Fields
/// - `base_liquidity`: The updated base token liquidity in the pool.
/// - `quote_liquidity`: The updated quote token liquidity in the pool.
/// - `protocol_fees_to_redeem`: The protocol fees collected from the swap.
/// - `providers_fees_to_redeem`: The providers fees collected from the swap.
/// - `amount_to_withdraw`: The amount of tokens to withdraw after the swap.
/// - `is_in_out`: Indicates whether the swap is "in-to-out" (true) or "out-to-in" (false).
#[derive(Debug)]
pub struct SwapPayload {
    base_liquidity: u64,
    quote_liquidity: u64,
    protocol_fees_to_redeem: u64,
    providers_fees_to_redeem: u64,
    amount_to_withdraw: u64,
    is_in_out: bool,
}

impl SwapPayload {
    /// Creates a new `SwapPayload` instance with the specified parameters.
    ///
    /// # Parameters
    /// - `base_liquidity`: The updated base token liquidity.
    /// - `quote_liquidity`: The updated quote token liquidity.
    /// - `protocol_fees_to_redeem`: The protocol fees collected during the swap.
    /// - `providers_fees_to_redeem`: The providers fees collected from the swap.
    /// - `amount_to_withdraw`: The amount of tokens withdrawn.
    /// - `is_in_out`: Indicates the direction of the swap.
    fn new(base_liquidity: u64, quote_liquidity: u64, protocol_fees_to_redeem: u64, providers_fees_to_redeem: u64, amount_to_withdraw: u64, is_in_out: bool) -> Self {
        Self{
            base_liquidity,
            quote_liquidity,
            protocol_fees_to_redeem,
            providers_fees_to_redeem,
            amount_to_withdraw,
            is_in_out,
        }
    }

    /// Returns the amount of tokens to withdraw after the swap.
    pub fn amount_to_withdraw(&self) -> u64{
        self.amount_to_withdraw
    }
}

/// Represents the data required for collecting protocol fees in the AMM.
///
/// This struct contains the protocol fees for redemption and left fees.
///
/// # Fields
/// - `protocol_base_fees_to_redeem`: The amount of protocol fees in base tokens for redemption.
/// - `protocol_quote_fees_to_redeem`: The amount of protocol fees in quote tokens for redemption.
/// - `new_protocol_base_fees_to_redeem`: Left amount of protocol fees in base tokens available for redemption.
/// - `new_protocol_quote_fees_to_redeem`: Left amount of protocol fees in quote tokens available for redemption.
#[derive(Debug)]
pub struct CollectFeesPayload {
    /// The amount of protocol fees in base tokens that will be redeemed.
    protocol_base_fees_to_redeem: u64,

    /// The amount of protocol fees in quote tokens that will be redeemed.
    protocol_quote_fees_to_redeem: u64,
    
    /// Left amount of protocol fees in base tokens that can be redeemed.
    new_protocol_base_fees_to_redeem: u64,

    /// Left amount of protocol fees in quote tokens that can be redeemed.
    new_protocol_quote_fees_to_redeem: u64,
}

impl CollectFeesPayload {
    /// Creates a new `CollectFeesPayload` instance with the specified parameters.
    ///
    /// # Parameters
    /// - `protocol_base_fees_to_redeem`: The amount of protocol fees in base tokens for redemption.
    /// - `protocol_quote_fees_to_redeem`: The amount of protocol fees in quote tokens for redemption.
    /// - `new_protocol_base_fees_to_redeem`: Left amount of protocol fees in base tokens available for redemption.
    /// - `new_protocol_quote_fees_to_redeem`: Left amount of protocol fees in quote tokens available for redemption.
    ///
    /// # Returns
    /// - A new instance of `CollectFeesPayload`.
    pub fn new(
        protocol_base_fees_to_redeem: u64,
        protocol_quote_fees_to_redeem: u64,
        new_protocol_base_fees_to_redeem: u64,
        new_protocol_quote_fees_to_redeem: u64,
    ) -> Self {
        Self {
            protocol_base_fees_to_redeem,
            protocol_quote_fees_to_redeem,
            new_protocol_base_fees_to_redeem,
            new_protocol_quote_fees_to_redeem
        }
    }

    /// Returns the amount of protocol fees in base tokens for redemption.
    pub fn protocol_base_fees_to_redeem(&self) -> u64 {
        self.protocol_base_fees_to_redeem
    }

    /// Returns the amount of protocol fees in quote tokens for redemption.
    pub fn protocol_quote_fees_to_redeem(&self) -> u64 {
        self.protocol_quote_fees_to_redeem
    }
}
#[cfg(test)]
mod payloads_tests {
    use super::*;

    /// Tests the `LaunchPayload` struct's creation and getters.
    #[test]
    fn test_launch_payload() {
        let payload = LaunchPayload::new(
            1000,
            Q64_128::from_u64(2000),
            Q64_128::from_u64(3000),
            4000,
            5000,
            6000,
        );

        assert_eq!(payload.initial_locked_liquidity, 1000);
        assert_eq!(payload.constant_product_sqrt, Q64_128::from_u64(2000));
        assert_eq!(payload.base_quote_ratio_sqrt, Q64_128::from_u64(3000));
        assert_eq!(payload.base_liquidity, 4000);
        assert_eq!(payload.quote_liquidity, 5000);
        assert_eq!(payload.lp_tokens_supply, 6000);

        assert_eq!(payload.initial_locked_liquidity(), 1000);
        assert_eq!(payload.launch_liquidity(), 5000);
    }

    /// Tests the `ProvidePayload` struct's creation and getters.
    #[test]
    fn test_provide_payload() {
        let payload = ProvidePayload::new(
            Q64_128::from_u64(2000),
            Q64_128::from_u64(3000),
            4000,
            5000,
            6000,
            7000,
        );

        assert_eq!(payload.base_quote_ratio_sqrt, Q64_128::from_u64(2000));
        assert_eq!(payload.constant_product, Q64_128::from_u64(3000));
        assert_eq!(payload.base_liquidity, 4000);
        assert_eq!(payload.quote_liquidity, 5000);
        assert_eq!(payload.lp_tokens_supply, 6000);
        assert_eq!(payload.lp_tokens_to_mint, 7000);

        assert_eq!(payload.lp_tokens_to_mint(), 7000);
    }

    /// Tests the `WithdrawPayload` struct's creation and getters.
    #[test]
    fn test_withdraw_payload() {
        let payload = WithdrawPayload::new(
            Q64_128::from_u64(2000),
            4000,
            5000,
            6000,
            1000,
            2000,
        );

        assert_eq!(payload.base_quote_ratio_sqrt, Q64_128::from_u64(2000));
        assert_eq!(payload.base_liquidity, 4000);
        assert_eq!(payload.quote_liquidity, 5000);
        assert_eq!(payload.lp_tokens_supply, 6000);
        assert_eq!(payload.base_withdraw_amount, 1000);
        assert_eq!(payload.quote_withdraw_amount, 2000);

        assert_eq!(payload.base_withdraw_amount(), 1000);
        assert_eq!(payload.quote_withdraw_amount(), 2000);
    }

    /// Tests the `SwapPayload` struct's creation and getters.
    #[test]
    fn test_swap_payload() {
        let payload = SwapPayload::new(4000, 5000, 6000, 6500,7000, true);

        assert_eq!(payload.base_liquidity, 4000);
        assert_eq!(payload.quote_liquidity, 5000);
        assert_eq!(payload.protocol_fees_to_redeem, 6000);
        assert_eq!(payload.providers_fees_to_redeem, 6500);
        assert_eq!(payload.amount_to_withdraw, 7000);
        assert!(payload.is_in_out);

        assert_eq!(payload.amount_to_withdraw(), 7000);
    }
    
    /// Tests the `CollectFeesPayload` struct's creation and getters.
    #[test]
    fn test_collect_fees_payload() {
        let payload = CollectFeesPayload::new(112314, 536454000, 0, 0);

        assert_eq!(payload.protocol_base_fees_to_redeem, 112314);
        assert_eq!(payload.protocol_quote_fees_to_redeem, 536454000);
        assert_eq!(payload.new_protocol_base_fees_to_redeem, 0);
        assert_eq!(payload.new_protocol_quote_fees_to_redeem, 0);
        assert_eq!(payload.protocol_base_fees_to_redeem(), 112314);
        assert_eq!(payload.protocol_quote_fees_to_redeem(), 536454000);
    }
}