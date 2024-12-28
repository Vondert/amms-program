use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Provided fee_rate for AmmsConfig exceeds 10000")]
    ConfigFeeRateExceeded,
}
