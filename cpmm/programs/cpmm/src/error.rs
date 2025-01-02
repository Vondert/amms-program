use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // AmmsConfig
    #[msg("The provided fee rate for AmmsConfig exceeds the maximum allowed value of 10000 basis points (100%).")]
    ConfigFeeRateExceeded,

    // CpAmm state errors
    #[msg("Quote liquidity is zero.")]
    QuoteLiquidityIsZero,

    #[msg("Base liquidity is zero.")]
    BaseLiquidityIsZero,

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

    #[msg("Calculated liquidity tokens to mint is zero.")]
    LpTokensToMintIsZero,

    #[msg("Base token withdrawal amount is zero.")]
    BaseWithdrawAmountIsZero,

    #[msg("Quote token withdrawal amount is zero.")]
    QuoteWithdrawAmountIsZero,

    #[msg("Post-fee swap amount is zero.")]
    PostfeeSwapAmountIsZero,

    #[msg("Swap result is zero.")]
    SwapResultIsZero,

    #[msg("Calculated slippage exceeds allowed tolerance.")]
    SwapSlippageExceeded,
    
    #[msg("Overflow error when providing liquidity.")]
    ProvideOverflowError,
    #[msg("Overflow error when withdrawing liquidity.")]
    WithdrawOverflowError,
    #[msg("Overflow error when swapping.")]
    SwapOverflowError,
    
    // CpAmm integrity errors
    #[msg("Liquidity token supply after withdrawal is zero. The pool cannot be drained completely.")]
    LpTokensLeftSupplyIsZero,

    #[msg("New quote liquidity is zero. Operation cannot proceed. The pool cannot be drained completely.")]
    NewQuoteLiquidityIsZero,

    #[msg("New base liquidity is zero. Operation cannot proceed. The pool cannot be drained completely.")]
    NewBaseLiquidityIsZero,

    #[msg("Constant product tolerance exceeded.")]
    ConstantProductToleranceExceeded,

    #[msg("Liquidity ratio tolerance exceeded.")]
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