use anchor_lang::prelude::*;
use crate::utils::math::Q64_128;
use crate::error::ErrorCode;

/// A trait for implementing core calculations and constants for a constant product automated market maker (AMM).
///
/// This trait defines methods and associated constants required for the operation of an AMM,
/// such as calculating square roots of the constant product, liquidity ratios, and handling fees.
/// The constants provide essential parameters like tolerance levels, initial liquidity, and fee structures.
pub(crate) trait CpAmmCalculate {
    /// The number of decimals for the initial LP token minting.
    ///
    /// Example:
    /// - `LP_MINT_INITIAL_DECIMALS = 5` means the initial locked LP tokens are calculated as `10^5`.
    const LP_MINT_INITIAL_DECIMALS: u8 = 5;

    /// The maximum allowable fee rate, expressed in basis points.
    ///
    /// - 1 basis point = 0.01%.
    /// - `FEE_MAX_BASIS_POINTS = 10000` corresponds to a maximum fee rate of 100%.
    const FEE_MAX_BASIS_POINTS: u128 = 10000;

    /// The minimum liquidity requirement for the AMM pool.
    ///
    /// - `MIN_LIQUIDITY = 5500` specifies the minimum amount of liquidity (in base units) required to
    ///   initialize or maintain the pool.
    const MIN_LIQUIDITY: u64 = 5500;

    /// The initial amount of locked LP tokens in the pool.
    ///
    /// - Calculated as `10^LP_MINT_INITIAL_DECIMALS`.
    /// - Example: If `LP_MINT_INITIAL_DECIMALS = 5`, then `INITIAL_LOCKED_LP_TOKENS = 100000`.
    const INITIAL_LOCKED_LP_TOKENS: u64 = 10_u64.pow(Self::LP_MINT_INITIAL_DECIMALS as u32);

    /// The tolerance for changes in the constant product during swaps.
    ///
    /// - This value is used to ensure the AMM adheres to the constant product rule with minimal deviation.
    /// - Defined as a `Q64_128` value representing a tolerance of `0.00001%`.
    const SWAP_CONSTANT_PRODUCT_TOLERANCE: Q64_128 = Q64_128::from_bits(0, 3402823669209384634633746074317);

    /// The tolerance for adjusting liquidity ratios.
    ///
    /// - Similar to `SWAP_CONSTANT_PRODUCT_TOLERANCE`, this constant defines the allowable deviation
    ///   when recalculating the liquidity ratio of the pool.
    /// - Defined as a `Q64_128` value representing a tolerance of `0.00001%`.
    const ADJUST_LIQUIDITY_RATIO_TOLERANCE: Q64_128 = Q64_128::from_bits(0, 3402823669209384634633746074317);

    /// Calculates the square root of the constant product for the AMM.
    ///
    /// # Returns
    /// - A `Q64_128` value representing the square root of the constant product.
    fn constant_product_sqrt(&self) -> Q64_128;

    /// Calculates the square root of the base-to-quote liquidity ratio.
    ///
    /// # Returns
    /// - A `Q64_128` value representing the square root of the ratio between base and quote liquidity.
    fn base_quote_ratio_sqrt(&self) -> Q64_128;

    /// Retrieves the base liquidity of the AMM pool.
    ///
    /// # Returns
    /// - A `u64` value representing the amount of base liquidity in the pool.
    fn base_liquidity(&self) -> u64;

    /// Retrieves the quote liquidity of the AMM pool.
    ///
    /// # Returns
    /// - A `u64` value representing the amount of quote liquidity in the pool.
    fn quote_liquidity(&self) -> u64;

    /// Retrieves the total supply of LP tokens in the pool.
    ///
    /// # Returns
    /// - A `u64` value representing the total supply of LP tokens.
    fn lp_tokens_supply(&self) -> u64;

    /// Retrieves the fee rate for liquidity providers, expressed in basis points.
    ///
    /// # Returns
    /// - A `u16` value representing the provider's fee rate in basis points.
    fn providers_fee_rate_basis_points(&self) -> u16;

    /// Retrieves the protocol fee rate, expressed in basis points.
    ///
    /// # Returns
    /// - A `u16` value representing the protocol's fee rate in basis points.
    fn protocol_fee_rate_basis_points(&self) -> u16;

    /// Calculates the amount of LP tokens to mint based on the provided liquidity.
    ///
    /// # Parameters
    /// - `new_constant_product_sqrt`: The square root of the new constant product after providing liquidity.
    ///
    /// # Returns
    /// - `Some(u64)` with the amount of LP tokens to mint if the calculation is valid.
    /// - `None` if the calculation fails (e.g., due to underflow or zero tokens).
    fn calculate_lp_mint_for_provided_liquidity(&self, new_constant_product_sqrt: Q64_128) -> Option<u64> {
        let provided_liquidity = new_constant_product_sqrt.checked_sub(self.constant_product_sqrt())?;

        let share_from_current_liquidity = provided_liquidity.checked_div(self.constant_product_sqrt())?;
        let tokens_to_mint = share_from_current_liquidity.checked_mul(Q64_128::from_u64(self.lp_tokens_supply()))?.as_u64();
        if tokens_to_mint == 0{
            return None;
        }
        Some(tokens_to_mint)
    }
    /// Calculates the amount of base and quote liquidity to withdraw for a given share of LP tokens.
    ///
    /// # Parameters
    /// - `lp_tokens`: The number of LP tokens being redeemed.
    ///
    /// # Returns
    /// - `Some((u64, u64))` with the base and quote liquidity amounts.
    /// - `None` if the calculation fails (e.g., due to zero tokens).
    fn calculate_liquidity_from_share(&self, lp_tokens: u64) -> Option<(u64, u64)>{
        if lp_tokens == 0{
            return None;
        }
        let liquidity_share = Q64_128::from_u64(lp_tokens).checked_div(Q64_128::from_u64(self.lp_tokens_supply()))?;
        let constant_product_sqrt_share = self.constant_product_sqrt().checked_mul(liquidity_share)?;

        let base_withdraw = constant_product_sqrt_share.saturating_mul(self.base_quote_ratio_sqrt()).as_u64();
        let quote_withdraw = constant_product_sqrt_share.saturating_checked_div(self.base_quote_ratio_sqrt())?.as_u64();
        
        if base_withdraw == 0 || quote_withdraw == 0{
            return None;
        }
        Some((base_withdraw, quote_withdraw))
    }

    /// Calculates the new base and quote liquidity after a swap.
    ///
    /// # Parameters
    /// - `swap_amount`: The amount being swapped.
    /// - `is_in_out`: Whether the swap is "in" (true) or "out" (false).
    ///
    /// # Returns
    /// - `Some((u64, u64))` with the new base and quote liquidity values.
    /// - `None` if the calculation fails.
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

    /// Validates and calculates the new liquidity ratio after liquidity change.
    ///
    /// # Parameters
    /// - `new_base_liquidity`: The new base liquidity.
    /// - `new_quote_liquidity`: The new quote liquidity.
    ///
    /// # Returns
    /// - `Ok(Q64_128)` with the new base-to-quote ratio square root.
    /// - `Err(ErrorCode)` if the new ratio exceeds the allowed tolerance.
    fn validate_and_calculate_liquidity_ratio(&self, new_base_liquidity: u64, new_quote_liquidity: u64) -> Result<Q64_128>{
        let new_base_quote_ratio_sqrt = Self::calculate_base_quote_ratio_sqrt(new_base_liquidity, new_quote_liquidity).ok_or(ErrorCode::BaseQuoteRatioCalculationFailed)?;
        let difference = self.base_quote_ratio_sqrt().abs_diff(new_base_quote_ratio_sqrt);
        let allowed_difference = self.base_quote_ratio_sqrt() * Self::ADJUST_LIQUIDITY_RATIO_TOLERANCE;
        require!(difference <= allowed_difference, ErrorCode::LiquidityRatioToleranceExceeded);
        Ok(new_base_quote_ratio_sqrt)
    }

    /// Validates the constant product after a swap.
    ///
    /// # Parameters
    /// - `new_base_liquidity`: The new base liquidity.
    /// - `new_quote_liquidity`: The new quote liquidity.
    ///
    /// # Returns
    /// - `Ok(())` if the constant product remains within tolerance.
    /// - `Err(ErrorCode)` if the product exceeds the allowed tolerance.
    fn validate_swap_constant_product(&self, new_base_liquidity: u64, new_quote_liquidity: u64) -> Result<()>{
        let new_constant_product_sqrt = Self::calculate_constant_product_sqrt(new_base_liquidity, new_quote_liquidity).ok_or(ErrorCode::ConstantProductCalculationFailed)?;
        let difference = self.constant_product_sqrt().abs_diff(new_constant_product_sqrt);
        let allowed_difference = self.constant_product_sqrt() * Self::SWAP_CONSTANT_PRODUCT_TOLERANCE;
        require!(difference <= allowed_difference, ErrorCode::ConstantProductToleranceExceeded);
        Ok(())
    }

    /// Calculates the protocol fee for a swap amount.
    ///
    /// # Parameters
    /// - `swap_amount`: The amount being swapped.
    ///
    /// # Returns
    /// - A `u64` representing the protocol fee.
    fn calculate_protocol_fee_amount(&self, swap_amount: u64) -> u64{
        ((swap_amount as u128) * (self.protocol_fee_rate_basis_points() as u128) / Self::FEE_MAX_BASIS_POINTS) as u64
    }
    /// Calculates the fee for liquidity providers for a swap amount.
    ///
    /// # Parameters
    /// - `swap_amount`: The amount being swapped.
    ///
    /// # Returns
    /// - A `u64` representing the providers' fee.
    fn calculate_providers_fee_amount(&self, swap_amount: u64) -> u64 {
        ((swap_amount as u128) * (self.providers_fee_rate_basis_points() as u128) / Self::FEE_MAX_BASIS_POINTS) as u64
    }

    /// Calculates the opposite liquidity value based on the constant product formula.
    ///
    /// # Parameters
    /// - `x_liquidity`: The current liquidity for one side (base or quote).
    ///
    /// # Returns
    /// - `Some(u64)` with the opposite liquidity value.
    /// - `None` if the result is zero.
    fn calculate_opposite_liquidity(&self, x_liquidity: u64) -> Option<u64> {
        let constant_product = self.constant_product_sqrt().square_as_u128();
        let opposite_liquidity = (constant_product / x_liquidity as u128) as u64;
        if opposite_liquidity == 0 {
            return None;
        }
        Some(opposite_liquidity)
    }

    /// Validates the result of a swap against the estimated result and allowed slippage.
    ///
    /// # Parameters
    /// - `swap_result`: The actual result of the swap.
    /// - `estimated_swap_result`: The estimated result of the swap.
    /// - `allowed_slippage`: The allowed slippage tolerance.
    ///
    /// # Returns
    /// - `Ok(())` if the swap result is within the allowed slippage.
    /// - `Err(ErrorCode)` if the result exceeds the slippage tolerance.
    fn check_swap_result(swap_result: u64, estimated_swap_result: u64, allowed_slippage: u64) -> Result<()> {
        require!(swap_result > 0, ErrorCode::SwapResultIsZero);
        require!(swap_result.abs_diff(estimated_swap_result) <= allowed_slippage, ErrorCode::SwapSlippageExceeded);
        Ok(())
    }
    
    /// Calculates the base-to-quote liquidity ratio square root.
    ///
    /// # Parameters
    /// - `base_liquidity`: The base liquidity.
    /// - `quote_liquidity`: The quote liquidity.
    ///
    /// # Returns
    /// - `Some(Q64_128)` with the ratio square root.
    /// - `None` if the ratio is zero.
    fn calculate_base_quote_ratio_sqrt(base_liquidity: u64, quote_liquidity: u64) -> Option<Q64_128> {
        let ratio = Q64_128::checked_div_sqrt(Q64_128::from_u64(base_liquidity), Q64_128::from_u64(quote_liquidity))?;
        if ratio.is_zero() {
            return None;
        }
        Some(ratio)
    }

    /// Calculates the square root of the constant product.
    ///
    /// # Parameters
    /// - `base_liquidity`: The base liquidity.
    /// - `quote_liquidity`: The quote liquidity.
    ///
    /// # Returns
    /// - `Some(Q64_128)` with the square root of the constant product.
    /// - `None` if the product is zero.
    fn calculate_constant_product_sqrt(base_liquidity: u64, quote_liquidity: u64) -> Option<Q64_128> {
        let constant_product_sqrt = Q64_128::sqrt_from_u128(base_liquidity as u128 * quote_liquidity as u128);
        if constant_product_sqrt.is_zero() {
            return None;
        }
        Some(constant_product_sqrt)
    }

    /// Calculates the initial LP token supply and locked liquidity during pool launch.
    ///
    /// # Parameters
    /// - `constant_product_sqrt`: The square root of the constant product for the pool.
    ///
    /// # Returns
    /// - `Ok((u64, u64))` with the initial LP token supply and locked liquidity.
    /// - `Err(ErrorCode)` if the supply is too small.
    fn calculate_launch_lp_tokens(constant_product_sqrt: Q64_128) -> Result<(u64, u64)> {
        let lp_tokens_supply = constant_product_sqrt.as_u64();
        require!(lp_tokens_supply > 0, ErrorCode::LpTokensCalculationFailed);
        let initial_locked_liquidity = Self::INITIAL_LOCKED_LP_TOKENS;
        let difference = lp_tokens_supply
            .checked_sub(initial_locked_liquidity)
            .ok_or(ErrorCode::LaunchLiquidityTooSmall)?;
        require!(difference >= initial_locked_liquidity << 3, ErrorCode::LaunchLiquidityTooSmall);
        Ok((lp_tokens_supply, initial_locked_liquidity))
    }
}

#[cfg(test)]
mod tests {
    use crate::state::cp_amm::CpAmmCalculate;
    use crate::utils::math::Q64_128;

    /// A helper struct for testing the `CpAmmCalculate` trait.
    struct TestCpAmm{
        base_liquidity: u64,
        quote_liquidity: u64,
        constant_product_sqrt: Q64_128,
        base_quote_ratio_sqrt: Q64_128,
        lp_tokens_supply: u64,
        providers_fee_rate_basis_points: u16,
        protocol_fee_rate_basis_points: u16,
    }
    
    impl TestCpAmm{
        /// Creates a new instance of `TestCpAmm` with calculated values.
        ///
        /// Returns `None` if any calculation fails.
        fn try_new(base_liquidity: u64, quote_liquidity: u64, providers_fee_rate_basis_points: u16, protocol_fee_rate_basis_points: u16) -> Option<Self>{
            let constant_product_sqrt = TestCpAmm::calculate_constant_product_sqrt(base_liquidity, quote_liquidity)?;
            let lp_tokens_supply = TestCpAmm::calculate_launch_lp_tokens(constant_product_sqrt).ok()?;
            let base_quote_ratio = TestCpAmm::calculate_base_quote_ratio_sqrt(base_liquidity, quote_liquidity)?;
            
            Some(
                Self{
                    base_liquidity,
                    quote_liquidity,
                    constant_product_sqrt,
                    base_quote_ratio_sqrt: base_quote_ratio,
                    lp_tokens_supply: lp_tokens_supply.0 + lp_tokens_supply.1,
                    providers_fee_rate_basis_points,
                    protocol_fee_rate_basis_points
                }
            )
        }
    }
    impl CpAmmCalculate for TestCpAmm {
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

    /// Unit tests for the `TestCpAmm` implementation.
    mod unit_tests {
        use super::*;
        
        /// Tests `calculate_base_quote_ratio_sqrt` with extreme values.
        #[test]
        fn test_calculate_base_quote_ratio_sqrt_extreme() {
            let zero_liquidity: u64 = 0;
            let min_liquidity: u64 = 1;
            let max_liquidity: u64 = u64::MAX;

            let result_min_max: f64 = TestCpAmm::calculate_base_quote_ratio_sqrt(min_liquidity, max_liquidity)
                .unwrap()
                .into();
            let result_max_min: f64 = TestCpAmm::calculate_base_quote_ratio_sqrt(max_liquidity, min_liquidity)
                .unwrap()
                .into();

            let expected_min_max: f64 = 5.421010862427522e-20;
            let expected_max_min: f64 = 1.8446744073709552e19;

            assert!(
                (expected_min_max.sqrt() - result_min_max).abs() < 1e-12,
                "Base-to-quote ratio mismatch for min-to-max liquidity. Expected: {}, Got: {}",
                expected_min_max.sqrt(),
                result_min_max
            );
            assert!(
                (expected_max_min.sqrt() - result_max_min).abs() < 1e-12,
                "Base-to-quote ratio mismatch for max-to-min liquidity. Expected: {}, Got: {}",
                expected_max_min.sqrt(),
                result_max_min
            );
            assert!(
                TestCpAmm::calculate_base_quote_ratio_sqrt(zero_liquidity, zero_liquidity).is_none(),
                "Expected None for zero liquidity in base-to-quote ratio calculation"
            );
        }
        /// Tests `calculate_base_quote_ratio_sqrt` with normal values.
        #[test]
        fn test_calculate_base_quote_ratio_sqrt() {
            let base_liquidity: u64 = 250;
            let quote_liquidity: u64 = 100;

            let ratio1: f64 = TestCpAmm::calculate_base_quote_ratio_sqrt(base_liquidity, quote_liquidity)
                .unwrap()
                .into();
            let ratio2: f64 = TestCpAmm::calculate_base_quote_ratio_sqrt(quote_liquidity, base_liquidity)
                .unwrap()
                .into();

            let expected_ratio1: f64 = 2.5;
            let expected_ratio2: f64 = 0.4;

            assert!(
                (expected_ratio1.sqrt() - ratio1).abs() < 1e-12,
                "Base-to-quote ratio mismatch for base > quote. Expected: {}, Got: {}",
                expected_ratio1.sqrt(),
                ratio1
            );
            assert!(
                (expected_ratio2.sqrt() - ratio2).abs() < 1e-12,
                "Base-to-quote ratio mismatch for quote > base. Expected: {}, Got: {}",
                expected_ratio2.sqrt(),
                ratio2
            );
        }
        
        /// Tests `calculate_constant_product_sqrt` with extreme values.
        #[test]
        fn test_calculate_constant_product_sqrt_extreme() {
            let zero_liquidity: u64 = 0;
            let min_liquidity: u64 = 1;
            let max_liquidity: u64 = u64::MAX;

            let result_max_max: f64 = TestCpAmm::calculate_constant_product_sqrt(max_liquidity, max_liquidity)
                .unwrap()
                .into();
            let result_min_min: f64 = TestCpAmm::calculate_constant_product_sqrt(min_liquidity, min_liquidity)
                .unwrap()
                .into();

            let expected_max_max: f64 = max_liquidity as f64;
            let expected_min_min: f64 = 1.0;

            assert!(
                TestCpAmm::calculate_constant_product_sqrt(zero_liquidity, max_liquidity).is_none(),
                "Expected None for zero liquidity in constant product calculation"
            );
            assert!(
                (expected_max_max - result_max_max).abs() < 1e-12,
                "Constant product mismatch for max-to-max liquidity. Expected: {}, Got: {}",
                expected_max_max,
                result_max_max
            );
            assert!(
                (expected_min_min - result_min_min).abs() < 1e-12,
                "Constant product mismatch for min-to-min liquidity. Expected: {}, Got: {}",
                expected_min_min,
                result_min_min
            );
        }
        /// Tests `calculate_constant_product_sqrt` with normal values.
        #[test]
        fn test_calculate_constant_product_sqrt() {
            let base_liquidity: u64 = 250;
            let quote_liquidity: u64 = 100;

            let expected_result = ((base_liquidity * quote_liquidity) as f64).sqrt();
            let result: f64 = TestCpAmm::calculate_constant_product_sqrt(base_liquidity, quote_liquidity)
                .unwrap()
                .into();

            assert!(
                (expected_result - result).abs() < 1e-12,
                "Constant product mismatch for normal liquidity. Expected: {}, Got: {}",
                expected_result,
                result
            );
        }

        /// Tests `calculate_launch_lp_tokens` for expected behavior.
        #[test]
        fn test_calculate_launch_lp_tokens() {
            let constant_product_sqrt = Q64_128::from_u64(543654623489);

            let (lp_tokens_supply, initial_locked_liquidity) =
                TestCpAmm::calculate_launch_lp_tokens(constant_product_sqrt).unwrap();

            let expected_lp_tokens_supply = constant_product_sqrt.as_u64();
            let expected_initial_locked_liquidity = 10_u64.pow(TestCpAmm::LP_MINT_INITIAL_DECIMALS as u32);

            assert_eq!(
                lp_tokens_supply, expected_lp_tokens_supply,
                "LP tokens supply mismatch. Expected: {}, Got: {}",
                expected_lp_tokens_supply,
                lp_tokens_supply
            );
            assert_eq!(
                initial_locked_liquidity, expected_initial_locked_liquidity,
                "Initial locked liquidity mismatch. Expected: {}, Got: {}",
                expected_initial_locked_liquidity,
                initial_locked_liquidity
            );
        }

        /// Tests `calculate_liquidity_from_share` for expected behavior.
        #[test]
        fn test_calculate_liquidity_from_share() {
            let base_liquidity: u64 = 1_000_000;
            let quote_liquidity: u64 = 20_000_000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 0, 0).unwrap();

            let lp_tokens = 2_000;
            let (base, quote) = amm.calculate_liquidity_from_share(lp_tokens).unwrap();

            assert!(
                base > 0 && quote > 0,
                "Liquidity shares should be positive, got base: {}, quote: {}",
                base,
                quote
            );

            let expected_base = ((lp_tokens as f64 / amm.lp_tokens_supply as f64) * amm.base_liquidity as f64).floor() as u64;
            let expected_quote = ((lp_tokens as f64 / amm.lp_tokens_supply as f64) * amm.quote_liquidity as f64).floor() as u64;

            assert_eq!(
                base, expected_base,
                "Base liquidity mismatch. Expected: {}, Got: {}",
                expected_base,
                base
            );
            assert_eq!(
                quote, expected_quote,
                "Quote liquidity mismatch. Expected: {}, Got: {}",
                expected_quote,
                quote
            );
        }

        /// Tests `calculate_lp_mint_for_provided_liquidity` for minting LP tokens.
        #[test]
        fn test_calculate_lp_mint_for_provided_liquidity() {
            let base_liquidity: u64 = 1_000_000;
            let quote_liquidity: u64 = 20_000_000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 0, 0).unwrap();

            let new_base_liquidity = base_liquidity + 1_500;
            let new_quote_liquidity = quote_liquidity + 3_000;

            let new_constant_product_sqrt = TestCpAmm::calculate_constant_product_sqrt(new_base_liquidity, new_quote_liquidity).unwrap();
            let minted_tokens = amm.calculate_lp_mint_for_provided_liquidity(new_constant_product_sqrt).unwrap();

            assert!(minted_tokens > 0, "Minted tokens should be positive, got: {}", minted_tokens);

            let expected_minted: f64 = (f64::from(new_constant_product_sqrt) - f64::from(amm.constant_product_sqrt))
                / f64::from(amm.constant_product_sqrt)
                * amm.lp_tokens_supply as f64;

            assert_eq!(
                minted_tokens,
                expected_minted.floor() as u64,
                "Minted tokens mismatch. Expected: {}, Got: {}",
                expected_minted.floor() as u64,
                minted_tokens
            );
        }

        /// Tests `calculate_protocol_fee_amount` for correctness.
        #[test]
        fn test_calculate_protocol_fee_amount() {
            let base_liquidity: u64 = 1_000_000;
            let quote_liquidity: u64 = 20_000_000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();

            let swap_amount = 10_000;
            let fee = amm.calculate_protocol_fee_amount(swap_amount);

            let expected_fee = (swap_amount as u128 * amm.protocol_fee_rate_basis_points() as u128 / 10_000) as u64;

            assert_eq!(
                fee, expected_fee,
                "Protocol fee mismatch. Expected: {}, Got: {}",
                expected_fee,
                fee
            );
        }

        /// Tests `calculate_providers_fee_amount` for correctness.
        #[test]
        fn test_calculate_providers_fee_amount() {
            let base_liquidity: u64 = 1_000_000;
            let quote_liquidity: u64 = 20_000_000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();

            let swap_amount = 10_000;
            let fee = amm.calculate_providers_fee_amount(swap_amount);

            let expected_fee = (swap_amount as u128 * amm.providers_fee_rate_basis_points() as u128 / 10_000) as u64;

            assert_eq!(
                fee, expected_fee,
                "Providers fee mismatch. Expected: {}, Got: {}",
                expected_fee,
                fee
            );
        }

        /// Tests `calculate_opposite_liquidity` for correctness.
        #[test]
        fn test_calculate_opposite_liquidity() {
            let base_liquidity: u64 = 1_000_000;
            let quote_liquidity: u64 = 20_000_000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();

            let x_liquidity = 52_334;
            let result = amm.calculate_opposite_liquidity(x_liquidity);

            let expected_opposite = (amm.base_liquidity() * amm.quote_liquidity()) / x_liquidity;

            assert_eq!(
                result.unwrap(), expected_opposite,
                "Opposite liquidity mismatch. Expected: {}, Got: {}",
                expected_opposite,
                result.unwrap()
            );
        }

        /// Tests `calculate_afterswap_liquidity` for expected behavior after a swap.
        #[test]
        fn test_calculate_afterswap_liquidity() {
            let base_liquidity: u64 = 1_000_000;
            let quote_liquidity: u64 = 20_000_000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();

            let swap_amount = 500;

            let (new_base_liquidity1, new_quote_liquidity1) = amm.calculate_afterswap_liquidity(swap_amount, true).unwrap();
            let (new_base_liquidity2, new_quote_liquidity2) = amm.calculate_afterswap_liquidity(swap_amount, false).unwrap();

            let expected_new_base1 = amm.base_liquidity() + swap_amount;
            let expected_new_quote1 = (amm.base_liquidity() * amm.quote_liquidity()) / expected_new_base1;

            let expected_new_quote2 = amm.quote_liquidity() + swap_amount;
            let expected_new_base2 = (amm.base_liquidity() * amm.quote_liquidity()) / expected_new_quote2;

            assert_eq!(
                new_base_liquidity1, expected_new_base1,
                "New base liquidity mismatch for swap in base. Expected: {}, Got: {}",
                expected_new_base1,
                new_base_liquidity1
            );
            assert_eq!(
                new_quote_liquidity1, expected_new_quote1,
                "New quote liquidity mismatch for swap in base. Expected: {}, Got: {}",
                expected_new_quote1,
                new_quote_liquidity1
            );
            assert_eq!(
                new_base_liquidity2, expected_new_base2,
                "New base liquidity mismatch for swap in quote. Expected: {}, Got: {}",
                expected_new_base2,
                new_base_liquidity2
            );
            assert_eq!(
                new_quote_liquidity2, expected_new_quote2,
                "New quote liquidity mismatch for swap in quote. Expected: {}, Got: {}",
                expected_new_quote2,
                new_quote_liquidity2
            );
        }

        /// Tests `validate_and_calculate_liquidity_ratio` for correct validation and calculation of liquidity ratio.
        #[test]
        fn test_validate_and_calculate_liquidity_ratio() {
            let base_liquidity: u64 = 1_000_000;
            let quote_liquidity: u64 = 20_000_000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();

            let new_base_liquidity = 2_000_000;
            let new_quote_liquidity = 40_000_000;
            let invalid_base_liquidity = 1_999_999;

            let ratio = amm.validate_and_calculate_liquidity_ratio(new_base_liquidity, new_quote_liquidity).unwrap();
            let invalid_ratio = amm.validate_and_calculate_liquidity_ratio(invalid_base_liquidity, new_quote_liquidity);

            assert!(
                invalid_ratio.is_err(),
                "Validation should fail for an invalid liquidity ratio. Got: {:?}",
                invalid_ratio
            );
            assert_eq!(
                ratio, amm.base_quote_ratio_sqrt,
                "Calculated liquidity ratio mismatch. Expected: {:?}, Got: {:?}",
                amm.base_quote_ratio_sqrt, ratio
            );
        }

        /// Tests `validate_swap_constant_product` for correct validation of constant product after a swap.
        #[test]
        fn test_validate_swap_constant_product() {
            let base_liquidity: u64 = 1_000_000;
            let quote_liquidity: u64 = 20_000_000;
            let amm = TestCpAmm::try_new(base_liquidity, quote_liquidity, 20, 100).unwrap();

            let new_base_liquidity = 5_000_000;
            let new_quote_liquidity = 4_000_000;
            let invalid_base_liquidity = 4_999_999;

            let result = amm.validate_swap_constant_product(new_base_liquidity, new_quote_liquidity);
            let invalid_result = amm.validate_swap_constant_product(invalid_base_liquidity, new_quote_liquidity);

            assert!(
                result.is_ok(),
                "Validation of constant product should pass for valid inputs. Got: {:?}",
                result
            );
            assert!(
                invalid_result.is_err(),
                "Validation of constant product should fail for invalid inputs. Got: {:?}",
                invalid_result
            );
        }

        /// Tests `check_swap_result` for correct validation of swap results within allowed slippage.
        #[test]
        fn test_check_swap_result() {
            let swap_result = 1_000;
            let estimated_swap_result = 1_020;
            let allowed_slippage = 25;

            let result = TestCpAmm::check_swap_result(swap_result, estimated_swap_result, allowed_slippage);
            assert!(
                result.is_ok(),
                "Swap result validation should pass within allowed slippage. Swap result: {}, Estimated: {}, Allowed Slippage: {}",
                swap_result, estimated_swap_result, allowed_slippage
            );

            let result = TestCpAmm::check_swap_result(swap_result, estimated_swap_result, 10);
            assert!(
                result.is_err(),
                "Swap result validation should fail if slippage is exceeded. Swap result: {}, Estimated: {}, Allowed Slippage: {}",
                swap_result, estimated_swap_result, 10
            );
        }
    }
    mod fuzz_tests {
        use super::*;
        use proptest::prelude::*;

        /// Generates arbitrary values for `u64`, including edge cases such as `MIN_LIQUIDITY`, `0`, and `u64::MAX`.
        fn arbitrary_u64() -> impl Strategy<Value = u64> {
            prop_oneof![
                0..=u64::MAX,
                Just(TestCpAmm::MIN_LIQUIDITY),
                Just(TestCpAmm::MIN_LIQUIDITY + 1),
                Just(TestCpAmm::MIN_LIQUIDITY + 2),
                Just(0),
                Just(1), 
                Just(2),
                Just(10), 
                Just(u64::MAX),
                Just(1 << 32),
            ]
        }
        
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10000))]

            /// Fuzz-test for `calculate_constant_product_sqrt` and `calculate_base_quote_ratio_sqrt`.
            /// Validates that results match the expected values, and edge cases like zero liquidity return `None`.
            #[test]
            fn test_fuzz_liquidity_calculations(base_liquidity in arbitrary_u64(), quote_liquidity in arbitrary_u64()) {
                let optional_constant_product_sqrt = TestCpAmm::calculate_constant_product_sqrt(base_liquidity, quote_liquidity);
                let optional_base_quote_ratio_sqrt = TestCpAmm::calculate_base_quote_ratio_sqrt(base_liquidity, quote_liquidity);
            
                if base_liquidity == 0 || quote_liquidity == 0 {
                    prop_assert!(
                        optional_constant_product_sqrt.is_none(),
                        "Constant product square root should return None when either liquidity is zero. Got: {:?}",
                        optional_constant_product_sqrt
                    );
                    prop_assert!(
                        optional_base_quote_ratio_sqrt.is_none(),
                        "Base-to-quote ratio square root should return None when either liquidity is zero. Got: {:?}",
                        optional_base_quote_ratio_sqrt
                    );
                } else if base_liquidity >= TestCpAmm::MIN_LIQUIDITY || quote_liquidity >= TestCpAmm::MIN_LIQUIDITY {
                    let constant_product_sqrt = optional_constant_product_sqrt.unwrap();
                    let base_quote_ratio_sqrt = optional_base_quote_ratio_sqrt.unwrap();
            
                    let restored_base = constant_product_sqrt.saturating_mul(base_quote_ratio_sqrt).as_u64();
                    let restored_quote = constant_product_sqrt.saturating_checked_div(base_quote_ratio_sqrt).unwrap().as_u64();
            
                    prop_assert!(
                        restored_base.abs_diff(base_liquidity) <= 1,
                        "Restored base liquidity exceeds tolerance. Expected: {}, Got: {}",
                        base_liquidity,
                        restored_base
                    );
                    prop_assert!(
                        restored_quote.abs_diff(quote_liquidity) <= 1,
                        "Restored quote liquidity exceeds tolerance. Expected: {}, Got: {}",
                        quote_liquidity,
                        restored_quote
                    );
                }
            }
            
            /// Fuzz-test for `calculate_launch_lp_tokens`.
            /// Ensures valid LP token calculations and edge case handling.
            #[test]
            fn test_fuzz_launch_lp_tokens_calculations(base_liquidity in arbitrary_u64(), quote_liquidity in arbitrary_u64()) {
                let optional_constant_product_sqrt = TestCpAmm::calculate_constant_product_sqrt(base_liquidity, quote_liquidity);
            
                if base_liquidity == 0 || quote_liquidity == 0 {
                    prop_assert!(
                        optional_constant_product_sqrt.is_none(),
                        "Constant product square root should return None when either liquidity is zero. Got: {:?}",
                        optional_constant_product_sqrt
                    );
                } else if base_liquidity >= TestCpAmm::MIN_LIQUIDITY || quote_liquidity >= TestCpAmm::MIN_LIQUIDITY {
                    let constant_product_sqrt = optional_constant_product_sqrt.unwrap();
                    let lp_tokens = constant_product_sqrt.as_u64();
            
                    if lp_tokens >> 3 >= TestCpAmm::INITIAL_LOCKED_LP_TOKENS {
                        let (launch_liquidity, initial_locked) = TestCpAmm::calculate_launch_lp_tokens(constant_product_sqrt).unwrap();
            
                        prop_assert_eq!(
                            launch_liquidity,
                            lp_tokens,
                            "Launch LP token calculation mismatch. Expected: {}, Got: {}. Initial Locked: {}",
                            lp_tokens,
                            launch_liquidity,
                            initial_locked
                        );
                        prop_assert!(
                            lp_tokens >> 3 >= initial_locked,
                            "Initial locked liquidity mismatch. LP Tokens: {}, Launch Liquidity: {}, Initial Locked: {}",
                            lp_tokens,
                            launch_liquidity,
                            initial_locked
                        );
                    }
                }
            }
        }
    }
}