use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{
    TokenAccount
};
use crate::state::{AmmsConfig, CpAmm};

#[derive(Accounts)]
pub struct LaunchCpAmm<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,
    pub base_mint: Box<Account<'info, Mint>>,
    pub quote_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub lp_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    // Token program will check mint and authority via transfer instruction
    pub signer_base_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    // Token program will check mint and authority via transfer instruction
    pub signer_quote_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = signer_lp_account.owner == signer.key()
    )]
    pub signer_lp_account: Box<Account<'info, token::TokenAccount>>,

    #[account(
        seeds = [AmmsConfig::SEED, amms_config.id.to_le_bytes().as_ref()],
        bump = amms_config.bump
    )]
    pub amms_config: Box<Account<'info, AmmsConfig>>,

    #[account(
        mut,
        constraint = !cp_amm.is_launched,
        constraint = amms_config.key() == cp_amm.amms_config,
        constraint = base_mint.key() == cp_amm.base_mint,
        constraint = quote_mint.key() == cp_amm.quote_mint,
        seeds = [CpAmm::SEED, lp_mint.key().as_ref()],
        bump = cp_amm.bump
    )]
    pub cp_amm: Box<Account<'info, CpAmm>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = base_mint,
        associated_token::authority = cp_amm,
    )]
    pub cp_amm_base_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = quote_mint,
        associated_token::authority = cp_amm,
    )]
    pub cp_amm_quote_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

/*impl<'info> LaunchCpAmm<'info>{
    fn get_provide_base_liquidity_transfer_context() ->{

    }
}*/

pub(crate) fn handler(ctx: Context<LaunchCpAmm>, base_launch_liquidity: u64, quote_launch_liquidity: u64) -> Result<()> {

    Ok(())
}