use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use crate::state::{AmmsConfig, AmmsConfigsManager};

#[derive(Accounts)]
pub struct UpdateAmmsConfigFeeAuthority<'info> {
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
    /// CHECK: New fee authority can be arbitrary
    new_fee_authority: UncheckedAccount<'info>,
}
pub(crate) fn handler(ctx: Context<UpdateAmmsConfigFeeAuthority>) -> Result<()> {
    ctx.accounts.amms_config.update_fee_authority(
        ctx.accounts.new_fee_authority.key()
    );
    Ok(())
}