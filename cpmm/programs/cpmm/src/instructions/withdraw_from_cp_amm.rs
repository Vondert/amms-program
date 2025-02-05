use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::{AmmsConfig, cp_amm::CpAmm};
use crate::utils::token_instructions::{BurnTokensInstructions, TransferTokensInstruction};

#[derive(Accounts)]
pub struct WithdrawFromCpAmm<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub lp_mint: Box<Account<'info, token::Mint>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = base_mint,
        associated_token::authority = signer,
        associated_token::token_program = base_token_program
    )]
    pub signer_base_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = quote_mint,
        associated_token::authority = signer,
        associated_token::token_program = quote_token_program
    )]
    pub signer_quote_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    // Token program will check mint and authority via token_instructions instruction
    pub signer_lp_account: Box<Account<'info, token::TokenAccount>>,

    #[account(
        seeds = [AmmsConfig::SEED, amms_config.id.to_le_bytes().as_ref()],
        bump = amms_config.bump()
    )]
    pub amms_config: Box<Account<'info, AmmsConfig>>,

    #[account(
        mut,
        constraint = cp_amm.is_launched(),
        constraint = amms_config.key() == cp_amm.amms_config().key(),
        constraint = lp_mint.key() == cp_amm.lp_mint,
        constraint = base_mint.key() == cp_amm.base_mint().key(),
        constraint = quote_mint.key() == cp_amm.quote_mint().key(),
        constraint = cp_amm_base_vault.key() == cp_amm.base_vault().key(),
        constraint = cp_amm_quote_vault.key() == cp_amm.quote_vault().key(),
        seeds = [CpAmm::SEED, cp_amm.lp_mint.as_ref()],
        bump = cp_amm.bump()
    )]
    pub cp_amm: Box<Account<'info, CpAmm>>,

    #[account(mut)]
    pub cp_amm_base_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub cp_amm_quote_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub lp_token_program: Program<'info, Token>,
    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<WithdrawFromCpAmm>, lp_tokens: u64) -> Result<()> {

    let withdraw_payload = ctx.accounts.cp_amm.get_withdraw_payload(lp_tokens)?;

    let withdraw_base_liquidity_instruction = Box::new(ctx.accounts.get_withdraw_base_liquidity_transfer_instruction(withdraw_payload.base_withdraw_amount())?);
    let withdraw_quote_liquidity_instruction = Box::new(ctx.accounts.get_withdraw_quote_liquidity_transfer_instruction(withdraw_payload.quote_withdraw_amount())?);

    let liquidity_burn_instruction = Box::new(ctx.accounts.get_liquidity_burn_instruction(lp_tokens));
    liquidity_burn_instruction.execute(None)?;

    let cp_amm_seeds = ctx.accounts.cp_amm.seeds();
    let withdraw_instruction_seeds: &[&[&[u8]]] = &[&cp_amm_seeds];

    withdraw_base_liquidity_instruction.execute(Some(withdraw_instruction_seeds))?;
    withdraw_quote_liquidity_instruction.execute(Some(withdraw_instruction_seeds))?;


    ctx.accounts.cp_amm.withdraw(withdraw_payload);

    Ok(())
}

impl<'info> WithdrawFromCpAmm<'info>{
    fn get_withdraw_base_liquidity_transfer_instruction(&self, base_liquidity: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::try_new(
            base_liquidity,
            &self.base_mint,
            self.cp_amm_base_vault.to_account_info(),
            self.cp_amm.to_account_info(),
            self.signer_base_account.to_account_info(),
            &self.base_token_program
        )
    }
    fn get_withdraw_quote_liquidity_transfer_instruction(&self, quote_liquidity: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::try_new(
            quote_liquidity,
            &self.quote_mint,
            self.signer_quote_account.to_account_info(),
            self.cp_amm.to_account_info(),
            self.cp_amm_quote_vault.to_account_info(),
            &self.quote_token_program
        )
    }
    fn get_liquidity_burn_instruction(&self, liquidity: u64) -> BurnTokensInstructions<'_, '_, '_, 'info>{
        BurnTokensInstructions::new(
            liquidity,
            &self.lp_mint,
            self.signer_lp_account.to_account_info(),
            self.signer.to_account_info(),
            &self.lp_token_program
        )
    }
}