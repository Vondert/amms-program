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
use crate::utils::token_instructions::{MintTokensInstructions, TransferTokensInstruction};

#[derive(Accounts)]
pub struct LaunchCpAmm<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub lp_mint: Box<Account<'info, token::Mint>>,
    #[account(mut)]
    // Token program will check mint and authority via token_instructions instruction
    pub signer_base_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    // Token program will check mint and authority via token_instructions instruction
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

    #[account(
        init,
        payer = signer,
        associated_token::mint = lp_mint,
        associated_token::authority = cp_amm,
    )]
    pub cp_amm_locked_lp_vault: Box<Account<'info, token::TokenAccount>>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> LaunchCpAmm<'info>{
    fn get_provide_base_liquidity_transfer_instruction(&self, base_liquidity: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::new(
            base_liquidity,
            &self.base_mint,
            &self.signer_base_account,
            self.signer.to_account_info(),
            &self.cp_amm_base_vault,
            &self.token_program,
            &self.token_2022_program
        )
    }
    fn get_provide_quote_liquidity_transfer_instruction(&self, quote_liquidity: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::new(
            quote_liquidity,
            &self.quote_mint,
            &self.signer_quote_account,
            self.signer.to_account_info(),
            &self.cp_amm_quote_vault,
            &self.token_program,
            &self.token_2022_program
        )
    }
    fn get_launch_liquidity_mint_instruction(&self, launch_liquidity: u64) -> Result<MintTokensInstructions<'_, '_, '_, 'info>>{
        MintTokensInstructions::new(
            launch_liquidity,
            &self.lp_mint,
            self.cp_amm.to_account_info(),
            &self.signer_lp_account,
            &self.token_program
        )
    }
    fn get_initial_locked_liquidity_mint_instruction(&self, initial_locked_liquidity: u64) -> Result<MintTokensInstructions<'_, '_, '_, 'info>>{
        MintTokensInstructions::new(
            initial_locked_liquidity,
            &self.lp_mint,
            self.cp_amm.to_account_info(),
            &self.cp_amm_locked_lp_vault,
            &self.token_program
        )
    }
}

pub(crate) fn handler(ctx: Context<LaunchCpAmm>, base_liquidity: u64, quote_liquidity: u64) -> Result<()> {
    
    let provide_base_liquidity_instruction = Box::new(ctx.accounts.get_provide_base_liquidity_transfer_instruction(base_liquidity)?);
    let provide_quote_liquidity_instruction = Box::new(ctx.accounts.get_provide_quote_liquidity_transfer_instruction(quote_liquidity)?);
    
    let base_liquidity_to_provide = provide_base_liquidity_instruction.get_amount_after_fee();
    let quote_liquidity_to_provide = provide_quote_liquidity_instruction.get_amount_after_fee();
    
    let launch_payload = ctx.accounts.cp_amm.get_launch_payload(base_liquidity_to_provide, quote_liquidity_to_provide)?;
    
    let launch_liquidity_mint_instruction = Box::new(ctx.accounts.get_launch_liquidity_mint_instruction(launch_payload.launch_liquidity())?);
    let initial_locked_liquidity_mint_instruction = Box::new(ctx.accounts.get_initial_locked_liquidity_mint_instruction(launch_payload.initial_locked_liquidity())?);
    
    provide_base_liquidity_instruction.execute(None)?;
    provide_quote_liquidity_instruction.execute(None)?;
    
    let cp_amm_seeds = ctx.accounts.cp_amm.seeds();
    let mint_instruction_seeds: &[&[&[u8]]] = &[&cp_amm_seeds];
    
    launch_liquidity_mint_instruction.execute(Some(mint_instruction_seeds))?;
    initial_locked_liquidity_mint_instruction.execute(Some(mint_instruction_seeds))?;

    ctx.accounts.cp_amm.launch(launch_payload, &ctx.accounts.cp_amm_base_vault, &ctx.accounts.cp_amm_quote_vault, &ctx.accounts.cp_amm_locked_lp_vault);
    
    Ok(())
}