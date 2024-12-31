use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Token};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{
    TokenAccount,
    Mint
};
use crate::state::{AmmsConfig, CpAmm};
use crate::utils::transfer::TokenTransferInstruction;

#[derive(Accounts)]
pub struct LaunchCpAmm<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub lp_mint: Box<Account<'info, token::Mint>>,
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
        constraint = !cp_amm.is_launched(),
        constraint = amms_config.key() == cp_amm.amms_config().key(),
        constraint = base_mint.key() == cp_amm.base_mint().key(),
        constraint = quote_mint.key() == cp_amm.quote_mint().key(),
        seeds = [CpAmm::SEED, lp_mint.key().as_ref()],
        bump = cp_amm.bump()
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

impl<'info> LaunchCpAmm<'info>{
    fn get_provide_base_liquidity_transfer_context(&self, base_liquidity: u64) -> Result<TokenTransferInstruction<'_, '_, '_, 'info>>{
        TokenTransferInstruction::new(
            base_liquidity,
            &self.base_mint,
            &self.signer_base_account,
            self.signer.to_account_info(),
            &self.cp_amm_base_vault,
            &self.token_program,
            &self.token_2022_program,
            None
        )
    }
    fn get_provide_quote_liquidity_transfer_context(&self, quote_liquidity: u64) -> Result<TokenTransferInstruction<'_, '_, '_, 'info>>{
        TokenTransferInstruction::new(
            quote_liquidity,
            &self.quote_mint,
            &self.signer_quote_account,
            self.signer.to_account_info(),
            &self.cp_amm_quote_vault,
            &self.token_program,
            &self.token_2022_program,
            None
        )
    }
}

pub(crate) fn handler(ctx: Context<LaunchCpAmm>, base_liquidity: u64, quote_liquidity: u64) -> Result<()> {
    
    let provide_base_instruction = Box::new(ctx.accounts.get_provide_base_liquidity_transfer_context(base_liquidity)?);
    let provide_quote_instruction = Box::new(ctx.accounts.get_provide_quote_liquidity_transfer_context(quote_liquidity)?);
    
    let base_liquidity_to_provide = provide_base_instruction.get_amount_after_fee();
    let quote_liquidity_to_provide = provide_quote_instruction.get_amount_after_fee();
    
    let launch_payload = ctx.accounts.cp_amm.get_launch_payload(base_liquidity_to_provide, quote_liquidity_to_provide)?;
    
    provide_base_instruction.execute_transfer()?;
    provide_quote_instruction.execute_transfer()?;
    
    // Mint lp tokens

    ctx.accounts.cp_amm.launch(launch_payload, &ctx.accounts.cp_amm_base_vault, &ctx.accounts.cp_amm_quote_vault);
    
    Ok(())
}