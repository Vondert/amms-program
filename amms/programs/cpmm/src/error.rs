use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // Accounts validation errors

    #[msg("Invalid CpAmm vault address: expected associated token account does not match.")]
    InvalidCpAmmVaultAddress,

    #[msg("CpAmm vault owner mismatch: the vault is not owned by the expected program.")]
    InvalidCpAmmVaultOwner,
    
    // AmmsConfig
    #[msg("The provided fee rate for AmmsConfig exceeds the maximum allowed value of 10000 basis points (100%).")]
    ConfigFeeRateExceeded,

    // CpAmm state errors
    #[msg("Quote liquidity is zero.")]
    QuoteLiquidityIsZero,

    #[msg("Base liquidity is zero.")]
    BaseLiquidityIsZero,
    
    #[msg("Quote liquidity is less then minimal operable liquidity.")]
    InsufficientQuoteLiquidity,

    #[msg("Base liquidity is less then minimal operable liquidity.")]
    InsufficientBaseLiquidity,

    #[msg("Liquidity tokens supply is zero.")]
    LpTokensSupplyIsZero,

    #[msg("CpAmm is not launched.")]
    CpAmmNotLaunched,

    #[msg("CpAmm is not initialized.")]
    CpAmmNotInitialized,

    #[msg("CpAmm is already initialized.")]
    CpAmmAlreadyInitialized,

    #[msg("CpAmm is already launched.")]
    CpAmmAlreadyLaunched,

    // CpAmm operations inputs errors
    #[msg("Provided quote liquidity is zero.")]
    ProvidedQuoteLiquidityIsZero,

    #[msg("Provided base liquidity is zero.")]
    ProvidedBaseLiquidityIsZero,

    #[msg("Provided liquidity tokens are zero.")]
    ProvidedLpTokensIsZero,

    #[msg("Swap amount cannot be zero.")]
    SwapAmountIsZero,

    #[msg("Estimated swap result cannot be zero.")]
    EstimatedResultIsZero,

    // CpAmm operations errors
    #[msg("Launch liquidity must be at least 4 times greater than the initial locked liquidity.")]
    LaunchLiquidityTooSmall,

    #[msg("Failed to calculate liquidity tokens to mint due to invalid input or overflow.")]
    LpTokensCalculationFailed,

    #[msg("Failed to calculate afterswap state due to invalid input or overflow.")]
    AfterswapCalculationFailed,
    
    #[msg("Failed to calculate withdraw liquidity due to invalid input or overflow.")]
    WithdrawLiquidityCalculationFailed,

    #[msg("Swap result is zero.")]
    SwapResultIsZero,
    
    #[msg("Swap fees are zero")]
    SwapFeesAreZero,
    
    #[msg("Calculated slippage exceeds allowed tolerance.")]
    SwapSlippageExceeded,
    
    #[msg("Overflow error when providing liquidity.")]
    ProvideOverflowError,
    #[msg("Overflow error when withdrawing liquidity.")]
    WithdrawOverflowError,
    #[msg("Overflow error when swapping.")]
    SwapOverflowError,
    
    #[msg("Protocol fees to redeem is zero")]
    ProvidersFeesIsZero,
    
    // CpAmm integrity errors
    #[msg("Failed to calculate base-to-quote liquidity ratio due to invalid input or overflow.")]
    BaseQuoteRatioCalculationFailed,

    #[msg("Failed to calculate constant product due to invalid input or overflow.")]
    ConstantProductCalculationFailed,
    
    #[msg("Constant product tolerance exceeded.")]
    ConstantProductToleranceExceeded,

    #[msg("Liquidity ratio tolerance exceeded.")]
    LiquidityRatioToleranceExceeded,
    
    
    

    #[msg("Tradable mint for CpAmm has freeze authority.")]
    MintHasFreezeAuthority,

    #[msg("Provided mint owned by unsupported token program.")]
    UnsupportedMint,

    #[msg("Tradable mint for CpAmm hase unsupported token extension.")]
    UnsupportedMintTokenExtension,

    #[msg("Mint with TransferFee extension failed to calculate fee")]
    MintTransferFeeCalculationFailed,
    
    #[msg("Insufficient balance in the token account to complete the transfer.")]
    InsufficientBalanceForTransfer,

    #[msg("Mint and Token Program mismatch")]
    MintAndTokenProgramMismatch,
    
    #[msg("Minting the requested amount of liquidity tokens cause supply overflow.")]
    LiquidityMintOverflow,

    #[msg("Burning the requested amount of liquidity tokens cause supply overflow.")]
    LiquidityBurnOverflow,
}