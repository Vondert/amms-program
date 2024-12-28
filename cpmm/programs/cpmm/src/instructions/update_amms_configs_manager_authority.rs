use anchor_lang::prelude::*;
use crate::constants::{AMMS_CONFIG_MANAGER_SEED};
use crate::state::AmmsConfigsManager;

#[derive(Accounts)]
pub struct UpdateAmmsConfigsManagerAuthority<'info> {
    #[account(
        mut,
        constraint = (authority.key() == amms_configs_manager.authority || authority.key() == amms_configs_manager.head_authority)
    )]
    authority: Signer<'info>,
    #[account(
        mut,
        seeds = [AMMS_CONFIG_MANAGER_SEED],
        bump = amms_configs_manager.bump
    )]
    amms_configs_manager: Account<'info, AmmsConfigsManager>,
    /// CHECK: New authority can be arbitrary
    new_authority: UncheckedAccount<'info>,
}
pub fn handler(ctx: Context<UpdateAmmsConfigsManagerAuthority>) -> Result<()> {
    ctx.accounts.amms_configs_manager.update_authority(
        ctx.accounts.new_authority.key()
    );
    Ok(())
}