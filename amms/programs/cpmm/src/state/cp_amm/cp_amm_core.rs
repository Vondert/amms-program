use crate::utils::math::Q64_128;

/// A trait defining the core parameters of a Constant Product Automated Market Maker.
pub trait CpAmmCore {
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
}