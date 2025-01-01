use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // AmmsConfig
    #[msg("The provided fee rate for AmmsConfig exceeds the maximum allowed value of 10000 basis points (100%).")]
    ConfigFeeRateExceeded,

    // CpAmm state errors
    #[msg("Quote liquidity in CpAmm is zero. Cannot perform swap operation.")]
    QuoteLiquidityIsZero,

    #[msg("Base liquidity in CpAmm is zero. Cannot perform swap operation.")]
    BaseLiquidityIsZero,

    #[msg("Liquidity tokens supply is zero.")]
    LpTokensSupplyIsZero,
    
    #[msg("CpAmm is not launched.")]
    CpAmmNotLaunched,

    #[msg("CpAmm is not initialized.")]
    CpAmmNotInitialized,

    #[msg("CpAmm has already been initialized.")]
    CpAmmAlreadyInitialized,

    #[msg("CpAmm has already been launched.")]
    CpAmmAlreadyLaunched,
    
    // CpAmm operations inputs errors
    #[msg("Provided quote liquidity is zero. Cannot perform operation.")]
    ProvidedQuoteLiquidityIsZero,

    #[msg("Provided base liquidity is zero. Cannot perform operation.")]
    ProvidedBaseLiquidityIsZero,

    #[msg("Provided liquidity tokens is zero. Cannot perform operation.")]
    ProvidedLpTokensIsZero,
    
    #[msg("The provided amount for the swap operation is zero. Please provide a positive value.")]
    SwapAmountIsZero,

    #[msg("Estimated result of the swap operation is zero. Please provide a positive value.")]
    EstimatedResultIsZero,
    
    // CpAmm operations errors
    #[msg("Launch liquidity must be 4 times bigger then initial locked liquidity.")]
    LaunchLiquidityTooSmall,

    #[msg("Liquidity tokens to mints is zero.")]
    LpTokensToMintIsZero,

    #[msg("Liquidity tokens left supply is zero. Withdraw operation will drain the pool.")]
    LpTokensLeftSupplyIsZero,
    
    #[msg("Zero base tokens cannot be withdrawn.")]
    BaseWithdrawAmountIsZero,

    #[msg("Zero quote tokens cannot be withdrawn.")]
    QuoteWithdrawAmountIsZero,

    #[msg("Postfee swap amount is zero. Cannot perform operation")]
    PostfeeSwapAmountIsZero,
    
    #[msg("Result of the swap operation is zero. Cannot perform operation.")]
    SwapResultIsZero,
    
    #[msg("The calculated slippage for the swap exceeds the allowed slippage tolerance.")]
    SwapSlippageExceeded,
    
    // CpAmm integrity errors
    #[msg("New quote liquidity for CpAmm is zero. Cannot perform operation.")]
    NewQuoteLiquidityIsZero,

    #[msg("New base liquidity for CpAmm is zero. Cannot perform operation.")]
    NewBaseLiquidityIsZero,
    
    #[msg("The calculated constant product exceeds the allowed tolerance.")]
    ConstantProductToleranceExceeded,

    #[msg("The calculated base-to-quote liquidity ratio exceeds the allowed tolerance.")]
    LiquidityRatioToleranceExceeded,
    


    #[msg("Tradable mint for CpAmm has freeze authority.")]
    MintHasFreezeAuthority,

    #[msg("Tradable mint for CpAmm owned by unsupported token program.")]
    UnsupportedTradableMint,

    #[msg("Tradable mint for CpAmm hase unsupported token extension.")]
    UnsupportedTokenExtension,

    #[msg("Mint with TransferFee extension failed to calculate fee")]
    MintTransferFeeCalculationFailed,
    
    #[msg("Insufficient balance in the token account to complete the token_instructions.")]
    InsufficientBalanceForTransfer,

    #[msg("Minting the requested amount of liquidity tokens cause supply overflow.")]
    LiquidityMintOverflow,

    #[msg("Burning the requested amount of liquidity tokens cause supply overflow.")]
    LiquidityBurnOverflow,
}