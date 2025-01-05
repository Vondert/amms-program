use anchor_lang::prelude::*;
use crate::utils::math::Q64_64;
use crate::error::ErrorCode;

pub(crate) trait CpAmmCalculate {
    const LP_MINT_INITIAL_DECIMALS: u8 = 5;
    // 0.00001% f64 = 0.0000001
    const SWAP_CONSTANT_PRODUCT_TOLERANCE: Q64_64 = Q64_64::new(1844674407371);
    // 0.00001% f64 = 0.0000001
    const ADJUST_LIQUIDITY_RATIO_TOLERANCE: Q64_64 = Q64_64::new(1844674407371);
    const FEE_MAX_BASIS_POINTS: u128 = 10000;
    fn constant_product_sqrt(&self) -> Q64_64;
    fn base_quote_ratio(&self) -> Q64_64;
    fn base_liquidity(&self) -> u64;
    fn quote_liquidity(&self) -> u64;
    fn lp_tokens_supply(&self) -> u64;
    fn providers_fee_rate_basis_points(&self) -> u16;
    fn protocol_fee_rate_basis_points(&self) -> u16;
    fn calculate_lp_mint_for_provided_liquidity(&self, new_constant_product_sqrt: Q64_64) -> Option<u64> {
        let provided_liquidity = new_constant_product_sqrt.checked_sub(self.constant_product_sqrt())?;

        let share_from_current_liquidity = provided_liquidity.checked_div(self.constant_product_sqrt())?;
        let tokens_to_mint = share_from_current_liquidity.checked_mul(Q64_64::from_u64(self.lp_tokens_supply()))?.to_u64();
        if tokens_to_mint == 0{
            return None;
        }
        Some(tokens_to_mint)
    }
    fn calculate_liquidity_from_share(&self, lp_tokens: u64) -> Option<(u64, u64)>{
        if lp_tokens == 0{
            return None;
        }
        let liquidity_share = Q64_64::from_u64(lp_tokens).checked_div(Q64_64::from_u64(self.lp_tokens_supply()))?;
        let constant_product_sqrt_share = self.constant_product_sqrt().checked_mul(liquidity_share)?;

        // Sqrt liquidity?
        let base_withdraw_square = constant_product_sqrt_share.checked_square_mul_as_u128(self.base_quote_ratio())?;
        let quote_withdraw_square = constant_product_sqrt_share.checked_square_div_as_u128(self.base_quote_ratio())?;
        
        let base_withdraw = Q64_64::sqrt_from_u128(base_withdraw_square).to_u64();
        let quote_withdraw = Q64_64::sqrt_from_u128(quote_withdraw_square).to_u64();
        
        if base_withdraw == 0 || quote_withdraw == 0{
            return None;
        }
        Some((base_withdraw, quote_withdraw))
    }
    fn calculate_afterswap_liquidity(&self, swap_amount: u64, is_in_out: bool) -> Option<(u64, u64)>{
        let mut new_base_liquidity = 0;
        let mut new_quote_liquidity = 0;
        if is_in_out {
            new_base_liquidity = self.base_liquidity().checked_add(swap_amount)?;
            new_quote_liquidity = self.calculate_opposite_liquidity(new_base_liquidity)?;
        }
        else{
            new_quote_liquidity = self.quote_liquidity().checked_add(swap_amount)?;
            new_base_liquidity = self.calculate_opposite_liquidity(new_quote_liquidity)?;
        }
        Some((new_base_liquidity, new_quote_liquidity))
    }

    fn validate_and_calculate_liquidity_ratio(&self, new_base_liquidity: u64, new_quote_liquidity: u64) -> Result<Q64_64>{
        let new_base_quote_ratio_sqrt = Self::calculate_base_quote_ratio(new_base_liquidity, new_quote_liquidity).ok_or(ErrorCode::BaseQuoteRatioCalculationFailed)?;
        let difference = self.base_quote_ratio().abs_diff(new_base_quote_ratio_sqrt);
        let allowed_difference = self.base_quote_ratio() * Self::ADJUST_LIQUIDITY_RATIO_TOLERANCE;
        require!(difference <= allowed_difference, ErrorCode::LiquidityRatioToleranceExceeded);
        Ok(new_base_quote_ratio_sqrt)
    }
    fn validate_swap_constant_product(&self, new_base_liquidity: u64, new_quote_liquidity: u64) -> Result<()>{
        let new_constant_product_sqrt = Self::calculate_constant_product_sqrt(new_base_liquidity, new_quote_liquidity).ok_or(ErrorCode::ConstantProductCalculationFailed)?;
        let difference = self.constant_product_sqrt().abs_diff(new_constant_product_sqrt);
        let allowed_difference = self.constant_product_sqrt() * Self::SWAP_CONSTANT_PRODUCT_TOLERANCE;
        require!(difference <= allowed_difference, ErrorCode::ConstantProductToleranceExceeded);
        Ok(())
    }
    fn calculate_protocol_fee_amount(&self, swap_amount: u64) -> u64{
        ((swap_amount as u128) * (self.protocol_fee_rate_basis_points() as u128) / Self::FEE_MAX_BASIS_POINTS) as u64
    }
    fn calculate_providers_fee_amount(&self, swap_amount: u64) -> u64{
        ((swap_amount as u128) * (self.providers_fee_rate_basis_points() as u128) / Self::FEE_MAX_BASIS_POINTS) as u64
    }
    fn calculate_opposite_liquidity(&self, x_liquidity: u64) -> Option<u64>{
        let opposite_liquidity = self.constant_product_sqrt().checked_square_div_as_u64(Q64_64::from_u64(x_liquidity))?;
        if opposite_liquidity == 0 {
            return None;
        }
        Some(opposite_liquidity)
    }

    fn check_swap_result(swap_result: u64, estimated_swap_result: u64, allowed_slippage:u64) -> Result<()>{
        require!(swap_result > 0, ErrorCode::SwapResultIsZero);
        require!(swap_result.abs_diff(estimated_swap_result) <= allowed_slippage, ErrorCode::SwapSlippageExceeded);
        Ok(())
    }
    fn calculate_base_quote_ratio(base_liquidity: u64, quote_liquidity: u64) -> Option<Q64_64>{
        if base_liquidity == 0 || quote_liquidity == 0 {
            return None
        }
        let ratio = Q64_64::from_u64(base_liquidity) / Q64_64::from_u64(quote_liquidity);
        if ratio.is_zero(){
            return None
        }
        Some(ratio)
    }
    fn calculate_constant_product_sqrt(base_liquidity: u64, quote_liquidity: u64) -> Option<Q64_64>{
        if base_liquidity == 0 || quote_liquidity == 0 {
            return None
        }
        let constant_product_sqrt = Q64_64::sqrt_from_u128(base_liquidity as u128 * quote_liquidity as u128);
        if constant_product_sqrt.is_zero(){
            return None
        }
        Some(constant_product_sqrt)
    }

    fn calculate_launch_lp_tokens(constant_product_sqrt: Q64_64) -> Result<(u64, u64)>{
        let lp_tokens_supply = constant_product_sqrt.to_u64();
        require!(lp_tokens_supply > 0, ErrorCode::LpTokensCalculationFailed);

        let initial_locked_liquidity = 10_u64.pow(Self::LP_MINT_INITIAL_DECIMALS as u32);

        let difference = lp_tokens_supply.checked_sub(initial_locked_liquidity).ok_or(ErrorCode::LaunchLiquidityTooSmall)?;
        require!(difference >= initial_locked_liquidity << 2, ErrorCode::LaunchLiquidityTooSmall);
        Ok((lp_tokens_supply, difference))
    }
}

#[cfg(test)]
mod tests {
    use crate::state::cp_amm::CpAmmCalculate;
    use crate::utils::math::Q64_64;

    struct TestCpAmm{
        base_liquidity: u64,
        quote_liquidity: u64,
        constant_product_sqrt: Q64_64,
        base_quote_ratio: Q64_64,
        lp_tokens_supply: u64,
        providers_fee_rate_basis_points: u16,
        protocol_fee_rate_basis_points: u16,
    }
    impl TestCpAmm{
        fn try_new(base_liquidity: u64, quote_liquidity: u64, providers_fee_rate: u16, protocol_fee_rate: u16) -> Option<Self>{
            let constant_product_sqrt = TestCpAmm::calculate_constant_product_sqrt(base_liquidity, quote_liquidity)?;
            let lp_tokens_supply = TestCpAmm::calculate_launch_lp_tokens(constant_product_sqrt).ok()?;
            let base_quote_ratio = TestCpAmm::calculate_base_quote_ratio(base_liquidity, quote_liquidity)?;
            
            Some(
                Self{
                    base_liquidity,
                    quote_liquidity,
                    constant_product_sqrt,
                    base_quote_ratio,
                    lp_tokens_supply: lp_tokens_supply.0 + lp_tokens_supply.1,
                    providers_fee_rate_basis_points: 0,
                    protocol_fee_rate_basis_points: 0,
                }
            )
        }
    }
    impl CpAmmCalculate for TestCpAmm {
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

    mod unit_tests {
        use super::*;
        #[test]
        fn test_calculate_none_base_quote_ratio_extreme() {
            let liquidity1: u64 = 0;
            let liquidity2: u64 = 0;
            let liquidity3: u64 = 1;
            let liquidity4: u64 = u64::MAX;
            let result1 = 5.421010862427522e-20;
            let result2 = 1.8446744073709552e19;
            assert_eq!(TestCpAmm::calculate_base_quote_ratio(liquidity3, liquidity4).unwrap().to_f64(), result1);
            assert_eq!(TestCpAmm::calculate_base_quote_ratio(liquidity4, liquidity3).unwrap().to_f64(), result2);
            assert!(TestCpAmm::calculate_base_quote_ratio(liquidity1, liquidity2).is_none());
        }
        #[test]
        fn test_calculate_base_quote_ratio_sqrt() {
            let liquidity1: u64 = 250;
            let liquidity2: u64 = 100;
            let result1 = 2.5;
            let result2 = 0.4;
            assert_eq!(TestCpAmm::calculate_base_quote_ratio(liquidity1, liquidity2).unwrap().to_f64(), result1);
            assert_eq!(TestCpAmm::calculate_base_quote_ratio(liquidity2, liquidity1).unwrap().to_f64(), result2);
        }
        #[test]
        fn test_calculate_constant_product_sqrt_extreme() {
            let liquidity1: u64 = 0;
            let liquidity2: u64 = u64::MAX;
            let liquidity3: u64 = 1;
            assert!(TestCpAmm::calculate_constant_product_sqrt(liquidity1, liquidity2).is_none());
            assert_eq!(TestCpAmm::calculate_constant_product_sqrt(liquidity2, liquidity2).unwrap().raw_value(), (u64::MAX as u128) << 64);
            assert_eq!(TestCpAmm::calculate_constant_product_sqrt(liquidity3, liquidity3).unwrap().raw_value(), 1 << 64);
        }
        #[test]
        fn test_calculate_constant_product_sqrt() {
            let liquidity1: u64 = 250;
            let liquidity2: u64 = 100;
            let result = ((liquidity1 * liquidity2) as f64).sqrt();
            assert_eq!(TestCpAmm::calculate_constant_product_sqrt(liquidity1, liquidity2).unwrap().to_f64(), result);
        }

        #[test]
        fn test_calculate_launch_lp_tokens() {

            let constant_product_sqrt = Q64_64::from_u64(543654623489);

            let result = TestCpAmm::calculate_launch_lp_tokens(constant_product_sqrt).unwrap();

            let expected_lp_tokens_supply = constant_product_sqrt.to_u64();
            let initial_locked_liquidity = 10_u64.pow(TestCpAmm::LP_MINT_INITIAL_DECIMALS as u32);
            let expected_difference = expected_lp_tokens_supply - initial_locked_liquidity;

            assert_eq!(result.0, expected_lp_tokens_supply, "LP tokens supply mismatch");
            assert_eq!(result.1, expected_difference, "Difference mismatch");
        }
        
        #[test]
        fn test_calculate_liquidity_from_share() {
            let base_liquidity: u64 = 1000000;
            let quote_liquidity: u64 = 20000000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 0, 0).unwrap();

            let lp_tokens = 2_000;
            let (base, quote) = amm.calculate_liquidity_from_share(lp_tokens).unwrap();

            assert!(base > 0 && quote > 0, "Liquidity shares should be positive");

            let expected_base = ((lp_tokens as f64 / amm.lp_tokens_supply as f64) * amm.base_liquidity as f64).floor() as u64;
            let expected_quote = ((lp_tokens as f64 / amm.lp_tokens_supply as f64) * amm.quote_liquidity as f64).floor() as u64;
            
            assert_eq!(base, expected_base, "Base liquidity mismatch");
            assert_eq!(quote, expected_quote, "Quote liquidity mismatch");
        }

        #[test]
        fn test_calculate_lp_mint_for_provided_liquidity() {
            let base_liquidity: u64 = 1000000;
            let quote_liquidity: u64 = 20000000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 0, 0).unwrap();
            let new_base_liquidity = 1000000 + 1500;
            let new_quote_liquidity = 20000000 + 3000;

            let new_constant_product_sqrt = TestCpAmm::calculate_constant_product_sqrt(new_base_liquidity, new_quote_liquidity).unwrap();
            let minted_tokens = amm.calculate_lp_mint_for_provided_liquidity(new_constant_product_sqrt).unwrap();

            assert!(minted_tokens > 0, "Minted tokens should be positive");

            let expected_minted = (new_constant_product_sqrt.to_f64() - amm.constant_product_sqrt.to_f64()) / amm.constant_product_sqrt.to_f64() * amm.lp_tokens_supply as f64;
            
            assert_eq!(minted_tokens, expected_minted.floor() as u64, "Minted tokens mismatch");
        }

        #[test]
        fn test_calculate_protocol_fee_amount() {
            let base_liquidity: u64 = 1000000;
            let quote_liquidity: u64 = 20000000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();

            let swap_amount = 10_000;
            let fee = amm.calculate_protocol_fee_amount(swap_amount);

            let expected_fee = (swap_amount as u128 * amm.protocol_fee_rate_basis_points() as u128 / 10_000) as u64;

            assert_eq!(fee, expected_fee, "Protocol fee mismatch");
        }

        #[test]
        fn test_calculate_providers_fee_amount() {
            let base_liquidity: u64 = 1000000;
            let quote_liquidity: u64 = 20000000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();

            let swap_amount = 10_000;
            let fee = amm.calculate_providers_fee_amount(swap_amount);

            let expected_fee = (swap_amount as u128 * amm.providers_fee_rate_basis_points() as u128 / 10_000) as u64;

            assert_eq!(fee, expected_fee, "Providers fee mismatch");
        }

        #[test]
        fn test_calculate_opposite_liquidity() {
            let base_liquidity: u64 = 1000000;
            let quote_liquidity: u64 = 20000000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();


            let x_liquidity = 52334;
            let result = amm.calculate_opposite_liquidity(x_liquidity);

            let expected_opposite = (amm.base_liquidity() * amm.quote_liquidity()) / x_liquidity;

            assert_eq!(result.unwrap(), expected_opposite, "Opposite liquidity mismatch");
        }
        
        #[test]
        fn test_calculate_afterswap_liquidity() {
            let base_liquidity: u64 = 1000000;
            let quote_liquidity: u64 = 20000000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();

            let swap_amount = 500;

            let (new_base_liquidity1, new_quote_liquidity1) = amm.calculate_afterswap_liquidity(swap_amount, true).unwrap();
            let (new_base_liquidity2, new_quote_liquidity2) = amm.calculate_afterswap_liquidity(swap_amount, false).unwrap();

            let expected_new_base1 = amm.base_liquidity() + swap_amount;
            let expected_new_quote1 = (amm.base_liquidity() * amm.quote_liquidity()) / expected_new_base1;

            let expected_new_quote2 = amm.quote_liquidity() + swap_amount;
            let expected_new_base2 = (amm.base_liquidity() * amm.quote_liquidity()) / expected_new_quote2;
            
            assert_eq!(new_base_liquidity1, expected_new_base1, "New base liquidity 1 mismatch");
            assert_eq!(new_quote_liquidity1, expected_new_quote1, "New quote liquidity 1 mismatch");
            assert_eq!(new_base_liquidity2, expected_new_base2, "New base liquidity 2 mismatch");
            assert_eq!(new_quote_liquidity2, expected_new_quote2, "New quote liquidity 2 mismatch");
        }

        #[test]
        fn test_validate_and_calculate_liquidity_ratio() {
            let base_liquidity: u64 = 1000000;
            let quote_liquidity: u64 = 20000000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();


            let new_base_liquidity = 2000000;
            let new_quote_liquidity = 40000000;
            let invalid_base_liquidity = 1999999;


            let ratio = amm.validate_and_calculate_liquidity_ratio(new_base_liquidity, new_quote_liquidity).unwrap();
            let invalid_ratio = amm.validate_and_calculate_liquidity_ratio(invalid_base_liquidity, new_quote_liquidity);
            
            assert!(invalid_ratio.is_err(), "This liquidity ratio must be invalid");
            assert_eq!(ratio, amm.base_quote_ratio, "Liquidity ratio mismatch");
        }

        #[test]
        fn test_validate_swap_constant_product() {
            let base_liquidity: u64 = 1000000;
            let quote_liquidity: u64 = 20000000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();


            let new_base_liquidity = 5000000;
            let new_quote_liquidity = 4000000;
            
            let invalid_base_liquidity = 4999999;
            
            let result = amm.validate_swap_constant_product(new_base_liquidity, new_quote_liquidity);
            let invalid_result = amm.validate_swap_constant_product(invalid_base_liquidity, new_quote_liquidity);
            assert!(result.is_ok(), "Validation of constant product should pass for valid inputs");
            assert!(invalid_result.is_err(), "Validation of constant product shouldn't pass for invalid inputs");
        }

        #[test]
        fn test_check_swap_result() {
            let swap_result = 1_000;
            let estimated_swap_result = 1_020;
            let allowed_slippage = 25;

            let result = TestCpAmm::check_swap_result(swap_result, estimated_swap_result, allowed_slippage);

            assert!(result.is_ok(), "Swap result validation should pass within allowed slippage");

            let result = TestCpAmm::check_swap_result(swap_result, estimated_swap_result, 10);

            assert!(result.is_err(), "Swap result validation should fail if slippage exceeded");
        }
        
    }
    mod fuzz_tests {
        use super::*;
        use proptest::prelude::*;
        fn arbitrary_u128() -> impl Strategy<Value = u128> {
            prop_oneof![
            0..=u128::MAX,
            Just(0),
            Just(1),
            Just(2),
            Just(u128::MAX),
            Just(u128::MAX / 2),
            Just(u128::MAX - 1),
            Just(1024),
            Just(4095),
            Just(8191),
            Just(10000),
            Just(12321),
            Just(65535),
            Just(12345),
            Just(54321),
            Just(99999),
            Just(45678),
            Just(87654),
            Just(10001),
            Just(9999),
            Just(2047),
            Just(65534)
            ]
        }
    }
}