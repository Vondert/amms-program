use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use crate::state::{AmmsConfig, AmmsConfigsManager};

#[derive(Accounts)]
pub struct UpdateAmmsConfigProtocolFeeRate<'info> {
    #[account(
        mut,
        constraint = (authority.key() == amms_configs_manager.authority().key() || authority.key() == amms_configs_manager.head_authority().key())
    )]
    authority: Signer<'info>,
    #[account(
        seeds = [AmmsConfigsManager::SEED],
        bump = amms_configs_manager.bump()
    )]
    amms_configs_manager: Account<'info, AmmsConfigsManager>,
    #[account(
        mut,
        seeds = [AmmsConfig::SEED, amms_config.id.to_le_bytes().as_ref()],
        bump = amms_config.bump()
    )]
    amms_config: Account<'info, AmmsConfig>,
}

pub(crate) fn handler(ctx: Context<UpdateAmmsConfigProtocolFeeRate>, new_protocol_fee_rate_basis_points: u16) -> Result<()> {
    ctx.accounts.amms_config.update_protocol_fee_rate(new_protocol_fee_rate_basis_points)
}