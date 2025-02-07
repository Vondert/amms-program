use anchor_lang::Accounts;
use anchor_lang::prelude::*;
use crate::constants::{AMMS_CONFIG_MANAGER_INITIALIZE_AUTHORITY_PUBKEY, ANCHOR_DISCRIMINATOR};
use crate::state::AmmsConfigsManager;
use crate::program::Cpmm;

#[derive(Accounts)]
pub struct InitializeAmmsConfigsManager<'info>{
    #[account(
        mut,
        constraint = program_data.upgrade_authority_address == Some(signer.key())
    )]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = ANCHOR_DISCRIMINATOR + AmmsConfigsManager::INIT_SPACE,
        seeds = [AmmsConfigsManager::SEED],
        bump
    )]
    pub amms_configs_manager: Account<'info, AmmsConfigsManager>,
    /// CHECK: Authority can be arbitrary
    pub authority: UncheckedAccount<'info>,
    /// CHECK: Signer will be head_authority on initialization
    #[account(constraint = program_data.upgrade_authority_address == Some(head_authority.key()))]
    pub head_authority: UncheckedAccount<'info>,
    pub program_data: Account<'info, ProgramData>,
    #[account(constraint = cpmm_program.programdata_address()? == Some(program_data.key()))]
    pub cpmm_program: Program<'info, Cpmm>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<InitializeAmmsConfigsManager>) -> Result<()> {
    ctx.accounts.amms_configs_manager.initialize(
        ctx.accounts.authority.key(),
        ctx.accounts.head_authority.key(),
        ctx.bumps.amms_configs_manager
    );
    Ok(())
}