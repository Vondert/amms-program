use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use crate::constants::{AMMS_CONFIG_MANAGER_INITIALIZE_AUTHORITY_PUBKEY, AMMS_CONFIG_MANAGER_SEED};
use crate::state::AmmsConfigsManager;

#[derive(Accounts)]
pub struct InitializeAmmsConfigsManager<'info>{
    #[account(mut, address = AMMS_CONFIG_MANAGER_INITIALIZE_AUTHORITY_PUBKEY)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + AmmsConfigsManager::INIT_SPACE,
        seeds = [AMMS_CONFIG_MANAGER_SEED],
        bump
    )]
    pub amms_configs_manager: Account<'info, AmmsConfigsManager>,
    /// CHECK: Authority can be arbitrary
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Signer will be head_authority on initialization
    #[account(address = AMMS_CONFIG_MANAGER_INITIALIZE_AUTHORITY_PUBKEY)]
    pub head_authority: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>
}

pub fn handler(ctx: Context<InitializeAmmsConfigsManager>) -> Result<()> {
    ctx.accounts.amms_configs_manager.initialize(
        ctx.accounts.authority.key(),
        ctx.accounts.head_authority.key(),
        ctx.bumps.amms_configs_manager
    );
    Ok(())
}