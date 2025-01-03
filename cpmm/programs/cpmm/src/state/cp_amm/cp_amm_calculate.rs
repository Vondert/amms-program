use anchor_lang::prelude::*;
use crate::utils::math::Q64_64;
use crate::error::ErrorCode;

pub(crate) trait CpAmmCalculate {
    const LP_MINT_INITIAL_DECIMALS: u8 = 5;
    // 0.0001% f64 = 0.000001
    const SWAP_CONSTANT_PRODUCT_TOLERANCE: Q64_64 = Q64_64::new(18446744073710);
    // 0.0001% f64 = 0.000001
    const ADJUST_LIQUIDITY_RATIO_TOLERANCE: Q64_64 = Q64_64::new(18446744073710);
    const FEE_MAX_BASIS_POINTS: u128 = 10000;
    fn constant_product_sqrt(&self) -> Q64_64;
    fn base_quote_ratio_sqrt(&self) -> Q64_64;
    fn base_liquidity(&self) -> u64;
    fn quote_liquidity(&self) -> u64;
    fn lp_tokens_supply(&self) -> u64;
    fn providers_fee_rate_basis_points(&self) -> u16;
    fn protocol_fee_rate_basis_points(&self) -> u16;
    fn calculate_launch_lp_tokens(constant_product_sqrt: Q64_64) -> Result<(u64, u64)>{
        let lp_tokens_supply = constant_product_sqrt.to_u64();
        require!(lp_tokens_supply > 0, ErrorCode::LpTokensCalculationFailed);

        let initial_locked_liquidity = 10_u64.pow(Self::LP_MINT_INITIAL_DECIMALS as u32);

        let difference = lp_tokens_supply.checked_sub(initial_locked_liquidity).ok_or(ErrorCode::LaunchLiquidityTooSmall)?;
        require!(difference >= initial_locked_liquidity << 2, ErrorCode::LaunchLiquidityTooSmall);
        Ok((lp_tokens_supply, difference))
    }
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

        let base_withdraw = constant_product_sqrt_share.checked_mul(self.base_quote_ratio_sqrt())?.to_u64();
        let quote_withdraw = constant_product_sqrt_share.checked_div(self.base_quote_ratio_sqrt())?.to_u64();
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
        let new_base_quote_ratio_sqrt = Self::calculate_base_quote_ratio_sqrt(new_base_liquidity, new_quote_liquidity).ok_or(ErrorCode::BaseQuoteRatioCalculationFailed)?;
        let difference = self.base_quote_ratio_sqrt().abs_diff(new_base_quote_ratio_sqrt);
        let allowed_difference = self.base_quote_ratio_sqrt() * Self::ADJUST_LIQUIDITY_RATIO_TOLERANCE;
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
        let opposite_liquidity = self.constant_product_sqrt().checked_div(Q64_64::sqrt_from_u128(x_liquidity as u128))?.square_as_u64();
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
    fn calculate_base_quote_ratio_sqrt(base_liquidity: u64, quote_liquidity: u64) -> Option<Q64_64>{
        if base_liquidity == 0 || quote_liquidity == 0 {
            return None
        }
        let ratio = (Q64_64::from_u64(base_liquidity) / (Q64_64::from_u64(quote_liquidity))).raw_value();
        let ratio_sqrt = Q64_64::sqrt_from_u128(ratio);
        if ratio_sqrt.is_zero(){
            return None
        }
        Some(ratio_sqrt)
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
}