use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The provided fee rate for AmmsConfig exceeds the maximum allowed value of 10000 basis points (100%).")]
    ConfigFeeRateExceeded,

    #[msg("Quote liquidity is zero. Cannot perform swap operation.")]
    QuoteLiquidityIsZero,

    #[msg("Base liquidity is zero. Cannot perform swap operation.")]
    BaseLiquidityIsZero,

    #[msg("The provided amount for the swap operation is zero. Please provide a positive value.")]
    SwapAmountIsZero,

    #[msg("The calculated slippage for the swap exceeds the allowed slippage tolerance.")]
    SwapSlippageExceeded,

    #[msg("The calculated constant product after the swap exceeds the allowed tolerance.")]
    SwapConstantProductToleranceExceeded,

    #[msg("Overflow occurred while updating liquidity values after the swap.")]
    UpdateAfterSwapOverflow,

    #[msg("The calculated base-to-quote liquidity ratio after liquidity adjustment exceeds the allowed tolerance.")]
    AdjustLiquidityRatioToleranceExceeded,
    
    #[msg("Provided total fee for CpAmm exceeds the maximum allowed value of 10000 basis points (100%).")]
    CpAmmFeeRateExceeded,

    #[msg("Tradable mint for CpAmm has freeze authority.")]
    MintHasFreezeAuthority,

    #[msg("Tradable mint for CpAmm owned by unsupported token program.")]
    UnsupportedTradableMint,

    #[msg("Tradable mint for CpAmm hase unsupported token extension.")]
    UnsupportedTokenExtension
}