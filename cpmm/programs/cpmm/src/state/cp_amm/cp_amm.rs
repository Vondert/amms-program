use anchor_lang::{account, InitSpace};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use anchor_spl::token_interface;
use crate::utils::math::Q64_128;
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
    constant_product_sqrt: Q64_128, // 16
    // Square root of the Base and Quote token's ration
    // Stored as square root in Q64.64 for computation accuracy 
    base_quote_ratio_sqrt: Q64_128, // 16
    
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
    fn constant_product_sqrt(&self) -> Q64_128 {
        self.constant_product_sqrt
    }

    fn base_quote_ratio_sqrt(&self) -> Q64_128 {
        self.base_quote_ratio_sqrt
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
    fn check_state(&self) -> Result<()> {
        require!(self.is_launched, ErrorCode::CpAmmNotLaunched);
        require!(self.quote_liquidity > 0, ErrorCode::BaseLiquidityIsZero);
        require!(self.base_liquidity > 0, ErrorCode::QuoteLiquidityIsZero);
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
    pub fn get_withdraw_payload(&self, lp_tokens: u64) -> Result<WithdrawPayload> {
        self.check_state()?;
        require!(lp_tokens > 0, ErrorCode::ProvidedLpTokensIsZero);

        let lp_tokens_left_supply = self.lp_tokens_supply.checked_sub(lp_tokens).ok_or(ErrorCode::WithdrawOverflowError)?;

        let (base_withdraw, quote_withdraw) = self.calculate_liquidity_from_share(lp_tokens).ok_or(ErrorCode::WithdrawLiquidityCalculationFailed)?;
        
        let new_base_liquidity = self.base_liquidity.checked_sub(base_withdraw).ok_or(ErrorCode::WithdrawOverflowError)?;
        let new_quote_liquidity = self.quote_liquidity.checked_sub(quote_withdraw).ok_or(ErrorCode::WithdrawOverflowError)?;
        println!("New base {} New quote {}", new_base_liquidity, new_quote_liquidity);
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
        self.base_quote_ratio_sqrt = launch_payload.base_quote_ratio_sqrt;
        self.base_vault = base_vault.key();
        self.quote_vault = quote_vault.key();
        self.locked_lp_vault = locked_lp_vault.key();
    }
    pub(crate) fn provide(&mut self, provide_payload: ProvidePayload) -> (){
        self.base_liquidity = provide_payload.base_liquidity;
        self.quote_liquidity = provide_payload.quote_liquidity;
        self.lp_tokens_supply = provide_payload.lp_tokens_supply;
        self.constant_product_sqrt = provide_payload.constant_product;
        self.base_quote_ratio_sqrt = provide_payload.base_quote_ratio_sqrt;
    }
    pub(crate) fn withdraw(&mut self, withdraw_payload: WithdrawPayload) -> (){
        self.base_liquidity = withdraw_payload.base_liquidity;
        self.quote_liquidity = withdraw_payload.quote_liquidity;
        self.lp_tokens_supply = withdraw_payload.lp_tokens_supply;
        self.constant_product_sqrt = Self::calculate_constant_product_sqrt(self.base_liquidity, self.quote_liquidity).unwrap();
        self.base_quote_ratio_sqrt = withdraw_payload.base_quote_ratio_sqrt;
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
        self.base_quote_ratio_sqrt = Self::calculate_base_quote_ratio_sqrt(self.base_liquidity, self.quote_liquidity).unwrap();
    }
}

#[cfg(test)]
mod cp_amm_tests {
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
        providers_fee_rate_basis_points: u16,
        protocol_fee_rate_basis_points: u16,
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

        fn providers_fee_rate_basis_points(mut self, value: u16) -> Self {
            self.providers_fee_rate_basis_points = value;
            self
        }

        fn protocol_fee_rate_basis_points(mut self, value: u16) -> Self {
            self.protocol_fee_rate_basis_points = value;
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
                providers_fee_rate_basis_points: self.providers_fee_rate_basis_points,
                protocol_fee_rate_basis_points: self.protocol_fee_rate_basis_points,
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
            .providers_fee_rate_basis_points(25)
            .protocol_fee_rate_basis_points(15)
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
        assert_eq!(amm.lp_mint(), &default_pubkey);
        assert_eq!(amm.base_vault(), &default_pubkey);
        assert_eq!(amm.quote_vault(), &default_pubkey);
        assert_eq!(amm.amms_config(), &default_pubkey);

        assert_eq!(amm.constant_product_sqrt(), Q64_128::from_u64(2000));
        assert_eq!(amm.base_quote_ratio_sqrt(), Q64_128::from_u64(3000));
        assert_eq!(amm.base_liquidity(), 4000);
        assert_eq!(amm.quote_liquidity(), 5000);
        assert_eq!(amm.lp_tokens_supply(), 6000);
        assert_eq!(amm.providers_fee_rate_basis_points(), 25);
        assert_eq!(amm.protocol_fee_rate_basis_points(), 15);
    }

    mod state_change_tests {
        use super::*;

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

        #[test]
        fn test_swap() {
            let mut amm = CpAmmBuilder::new().build();

            let swap_payload_in = SwapPayload::new(4000, 1000, 1, 100, true);
            let swap_payload_out = SwapPayload::new(1000, 1000, 15, 100, false);

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
    }
    
    mod operations_calculations_tests {
        use super::*;
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
        ///
        /// Ensures that the method correctly calculates the updated state of the pool when
        /// additional liquidity is provided. Validates the updated base and quote liquidity,
        /// the new liquidity ratio, and the LP tokens minted.
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
        /*
                #[test]
                fn test_get_base_to_quote_swap_payload() {
                    let amm = CpAmmBuilder::new()
                        .is_launched(true)
                        .base_liquidity(1000)
                        .quote_liquidity(2000)
                        .protocol_base_fees_to_redeem(0)
                        .build();
        
                    let base_amount = 100;
                    let estimated_result = 190;
                    let allowed_slippage = 10;
        
                    let payload = amm
                        .get_base_to_quote_swap_payload(base_amount, estimated_result, allowed_slippage)
                        .unwrap();
        
                    assert!(payload.base_liquidity > 0);
                    assert!(payload.quote_liquidity > 0);
                    assert!(payload.protocol_fees_to_redeem > 0);
                    assert!(payload.amount_to_withdraw > 0);
                    assert_eq!(payload.is_in_out, true);
                }
        
                #[test]
                fn test_get_quote_to_base_swap_payload() {
                    let amm = CpAmmBuilder::new()
                        .is_launched(true)
                        .base_liquidity(1000)
                        .quote_liquidity(2000)
                        .protocol_quote_fees_to_redeem(0)
                        .build();
        
                    let quote_amount = 200;
                    let estimated_result = 90;
                    let allowed_slippage = 10;
        
                    let payload = amm
                        .get_quote_to_base_swap_payload(quote_amount, estimated_result, allowed_slippage)
                        .unwrap();
        
                    assert!(payload.base_liquidity > 0);
                    assert!(payload.quote_liquidity > 0);
                    assert!(payload.protocol_fees_to_redeem > 0);
                    assert!(payload.amount_to_withdraw > 0);
                    assert_eq!(payload.is_in_out, false);
                }*/
    }
}

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
    pub fn initial_locked_liquidity(&self) -> u64{
        self.initial_locked_liquidity
    }
    pub fn launch_liquidity(&self) -> u64{
        self.lp_tokens_supply.checked_sub(self.initial_locked_liquidity).unwrap()
    }
}

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
    pub fn lp_tokens_to_mint(&self) -> u64{
        self.lp_tokens_to_mint
    }
}

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

#[cfg(test)]
mod payloads_tests {
    use super::*;

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

    #[test]
    fn test_swap_payload() {
        let payload = SwapPayload::new(4000, 5000, 6000, 7000, true);

        assert_eq!(payload.base_liquidity, 4000);
        assert_eq!(payload.quote_liquidity, 5000);
        assert_eq!(payload.protocol_fees_to_redeem, 6000);
        assert_eq!(payload.amount_to_withdraw, 7000);
        assert!(payload.is_in_out);

        assert_eq!(payload.amount_to_withdraw(), 7000);
    }
}