use anchor_lang::prelude::*;
use crate::error::ErrorCode;
use anchor_spl::{
    token::{ID as TOKEN_PROGRAM_ID},
    token_2022::{ID as TOKEN_2022_PROGRAM_ID},
    token_interface::Mint
};
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022::spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions};
use crate::utils::math::Q64_64;

const ALLOWED_TOKEN_EXTENSIONS: &[ExtensionType] = &[
    ExtensionType::TransferFeeConfig,
    ExtensionType::ImmutableOwner,
    ExtensionType::ConfidentialTransferMint,
    ExtensionType::MemoTransfer,
    ExtensionType::InterestBearingConfig,
    ExtensionType::TransferHook,
    ExtensionType::MetadataPointer,
    ExtensionType::TokenMetadata,
    ExtensionType::GroupPointer,
    ExtensionType::TokenGroup,
    ExtensionType::GroupMemberPointer,
    ExtensionType::TokenGroupMember
];

pub(crate) fn validate_tradable_mint(tradable_mint: &InterfaceAccount<Mint>) -> Result<()>{
    require!(tradable_mint.freeze_authority.is_none(), ErrorCode::MintHasFreezeAuthority);
    let mint_account_info = tradable_mint.to_account_info();
    if mint_account_info.owner.key() == TOKEN_PROGRAM_ID{
        return Ok(())
    }
    if mint_account_info.owner.key() == TOKEN_2022_PROGRAM_ID{
        require!(StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_account_info.data.borrow()).map_err(|_| ProgramError::InvalidAccountData)?
            .get_extension_types()?.iter().all(
                |x| ALLOWED_TOKEN_EXTENSIONS.contains(x)
            ), ErrorCode::UnsupportedTokenExtension
        );
        return Ok(())
    }
    Err(ErrorCode::UnsupportedTradableMint.into())
}

pub(crate) fn check_swap_result(swap_result: u64, estimated_swap_result: u64, allowed_slippage:u64) -> Result<()>{
    require!(swap_result > 0, ErrorCode::SwapResultIsZero);
    require!(swap_result.abs_diff(estimated_swap_result) <= allowed_slippage, ErrorCode::SwapSlippageExceeded);
    Ok(())
}
pub(crate) fn calculate_base_quote_ratio_sqrt(base_liquidity: u64, quote_liquidity: u64) -> Q64_64{
    Q64_64::sqrt_from_u128((Q64_64::from_u64(base_liquidity) / Q64_64::from_u64(quote_liquidity)).raw_value())
}
pub(crate) fn calculate_constant_product_sqrt(base_liquidity: u64, quote_liquidity: u64) -> Q64_64{
    Q64_64::sqrt_from_u128(base_liquidity as u128 * quote_liquidity as u128)
}