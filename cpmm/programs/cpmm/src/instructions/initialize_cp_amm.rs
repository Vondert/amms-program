use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{AmmsConfig, CpAmm};

#[derive(Accounts)]
pub struct InitializeCpAmm<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        constraint = base_mint.freeze_authority.is_none()
    )]
    pub base_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = base_mint.key() != quote_mint.key(),
        constraint = quote_mint.freeze_authority.is_none()
    )]
    pub quote_mint: Box<Account<'info, Mint>>,
    
    #[account(
        init,
        payer = signer,
        mint::decimals = 6,
        mint::authority = cp_amm,
        mint::token_program = token_program
    )]
    /// Check freeze authority on client
    pub lp_mint: Box<Account<'info, Mint>>,
    
    #[account(
        seeds = [AmmsConfig::SEED, amms_config.id.to_le_bytes().as_ref()],
        bump = amms_config.bump
    )]
    pub amms_config: Box<Account<'info, AmmsConfig>>,
    
    #[account(
        init,
        payer = signer,
        space = 8 + CpAmm::INIT_SPACE,
        seeds = [CpAmm::SEED, lp_mint.key().as_ref()],
        bump
    )]
    pub cp_amm: Box<Account<'info, CpAmm>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = lp_mint,
        associated_token::authority = signer,
    )]
    pub signer_lp_token_account: Account<'info, TokenAccount>, 
    
    /*   #[account(
         init,
         payer = signer,
         associated_token::mint = base_mint,
         associated_token::authority = cp_amm
     )]
   pub base_vault: Account<'info, TokenAccount>,
   #[account(
         init,
         payer = signer,
         associated_token::mint = quote_mint,
         associated_token::authority = cp_amm
     )]
     pub quote_vault: Account<'info, TokenAccount>,*/
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


pub(crate) fn handler(ctx: Context<InitializeCpAmm>) -> Result<()> {
    ctx.accounts.cp_amm.initialize(
        &ctx.accounts.base_mint,
        &ctx.accounts.quote_mint,
        &ctx.accounts.lp_mint,
        &ctx.accounts.amms_config,
        ctx.bumps.cp_amm
    );
    Ok(())
}