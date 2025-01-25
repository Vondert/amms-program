use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    token_interface
};
use crate::state::{AmmsConfig, cp_amm::{
    CpAmm, 
    CpAmmCalculate
}};
use crate::utils::validate_tradable_mint;

#[derive(Accounts)]
pub struct InitializeCpAmm<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub base_mint: Box<InterfaceAccount<'info, token_interface::Mint>>,
    #[account(
        constraint = base_mint.key() != quote_mint.key()
    )]
    pub quote_mint: Box<InterfaceAccount<'info, token_interface::Mint>>,
    
    #[account(
        init,
        payer = signer,
        mint::decimals = CpAmm::LP_MINT_INITIAL_DECIMALS,
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
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> InitializeCpAmm<'info>{
    fn validate_base_mint(&self) -> Result<()> {
        let base_mint = self.base_mint.as_ref();
        validate_tradable_mint(base_mint)
    }
    fn validate_quote_mint(&self) -> Result<()> {
        let quote_mint = self.quote_mint.as_ref();
        validate_tradable_mint(quote_mint)
    }
}

pub(crate) fn handler(ctx: Context<InitializeCpAmm>) -> Result<()> {
    let accounts = ctx.accounts;
    
    accounts.validate_base_mint()?;
    accounts.validate_quote_mint()?;
    
    accounts.cp_amm.initialize(
        &accounts.base_mint,
        &accounts.quote_mint,
        &accounts.lp_mint,
        &accounts.amms_config,
        ctx.bumps.cp_amm
    )
}