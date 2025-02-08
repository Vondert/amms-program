use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{Token, TokenAccount},
    token_interface::{TokenAccount as InterfaceTokenAccount, Mint, TokenInterface}
};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{AmmsConfig, cp_amm::CpAmm};
use crate::utils::{
    token_instructions::{MintTokensInstructions, TransferTokensInstruction}
};

#[derive(Accounts)]
pub struct LaunchCpAmm<'info>{
    #[account(mut)]
    pub creator: Signer<'info>,
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub lp_mint: Box<Account<'info, token::Mint>>,
    #[account(mut)]
    // Token program will check mint and authority via token_instructions instruction
    pub creator_base_account: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,
    #[account(mut)]
    // Token program will check mint and authority via token_instructions instruction
    pub creator_quote_account: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = lp_mint,
        associated_token::authority = creator,
        associated_token::token_program = lp_token_program,
    )]
    pub creator_lp_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [AmmsConfig::SEED, amms_config.id.to_le_bytes().as_ref()],
        bump = amms_config.bump()
    )]
    pub amms_config: Box<Account<'info, AmmsConfig>>,

    #[account(
        mut,
        constraint = !cp_amm.is_launched(),
        constraint = creator.key() == cp_amm.creator().key(),
        constraint = amms_config.key() == cp_amm.amms_config().key(),
        constraint = lp_mint.key() == cp_amm.lp_mint,
        constraint = base_mint.key() == cp_amm.base_mint().key(),
        constraint = quote_mint.key() == cp_amm.quote_mint().key(),
        constraint = cp_amm_locked_lp_vault.key() == cp_amm.locked_lp_vault().key(),
        constraint = cp_amm_base_vault.key() == cp_amm.base_vault().key(),
        constraint = cp_amm_quote_vault.key() == cp_amm.quote_vault().key(),
        seeds = [CpAmm::SEED, cp_amm.lp_mint.as_ref()],
        bump = cp_amm.bump()
    )]
    pub cp_amm: Box<Account<'info, CpAmm>>,
    
    #[account(
        mut,
        seeds = [CpAmm::VAULT_SEED, cp_amm.key().as_ref(), cp_amm.base_mint().as_ref()],
        bump = cp_amm.base_vault_bump()
    )]
    pub cp_amm_base_vault: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,

    #[account(
        mut,
        seeds = [CpAmm::VAULT_SEED, cp_amm.key().as_ref(), cp_amm.quote_mint().as_ref()],
        bump = cp_amm.quote_vault_bump()
    )]
    pub cp_amm_quote_vault: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,

    #[account(
        mut,
        seeds = [CpAmm::VAULT_SEED, cp_amm.key().as_ref(), cp_amm.lp_mint.as_ref()],
        bump = cp_amm.locked_lp_vault_bump()
    )]
    pub cp_amm_locked_lp_vault: Box<Account<'info, TokenAccount>>,
    
    pub lp_token_program: Program<'info, Token>,
    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn handler(ctx: Context<LaunchCpAmm>, base_liquidity: u64, quote_liquidity: u64) -> Result<()> {
    
    let provide_base_liquidity_instruction = Box::new(ctx.accounts.get_provide_base_liquidity_transfer_instruction(base_liquidity)?);
    let provide_quote_liquidity_instruction = Box::new(ctx.accounts.get_provide_quote_liquidity_transfer_instruction(quote_liquidity)?);

    let base_liquidity_to_provide = provide_base_liquidity_instruction.get_amount_after_fee();
    let quote_liquidity_to_provide = provide_quote_liquidity_instruction.get_amount_after_fee();

    let launch_payload = Box::new(ctx.accounts.cp_amm.get_launch_payload(base_liquidity_to_provide, quote_liquidity_to_provide)?);

    let launch_liquidity_mint_instruction = Box::new(ctx.accounts.get_launch_liquidity_mint_instruction(launch_payload.launch_liquidity()));
    let initial_locked_liquidity_mint_instruction = Box::new(ctx.accounts.get_initial_locked_liquidity_mint_instruction(launch_payload.initial_locked_liquidity()));

    provide_base_liquidity_instruction.execute(None)?;
    provide_quote_liquidity_instruction.execute(None)?;

    let cp_amm_seeds = ctx.accounts.cp_amm.seeds();
    let mint_instruction_seeds: &[&[&[u8]]] = &[&cp_amm_seeds];

    launch_liquidity_mint_instruction.execute(Some(mint_instruction_seeds))?;
    initial_locked_liquidity_mint_instruction.execute(Some(mint_instruction_seeds))?;

    ctx.accounts.cp_amm.launch(*launch_payload);
    Ok(())
}

impl<'info> LaunchCpAmm<'info>{
    #[inline(never)]
    fn get_provide_base_liquidity_transfer_instruction(&self, base_liquidity: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::try_new(
            base_liquidity,
            &self.base_mint,
            self.creator_base_account.to_account_info(),
            self.creator.to_account_info(),
            self.cp_amm_base_vault.to_account_info(),
            &self.base_token_program
        )
    }

    #[inline(never)]
    fn get_provide_quote_liquidity_transfer_instruction(&self, quote_liquidity: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::try_new(
            quote_liquidity,
            &self.quote_mint,
            self.creator_quote_account.to_account_info(),
            self.creator.to_account_info(),
            self.cp_amm_quote_vault.to_account_info(),
            &self.quote_token_program
        )
    }

    #[inline(never)]
    fn get_launch_liquidity_mint_instruction(&self, launch_liquidity: u64) -> MintTokensInstructions<'_, '_, '_, 'info>{
        MintTokensInstructions::new(
            launch_liquidity,
            &self.lp_mint,
            self.cp_amm.to_account_info(),
            self.creator_lp_account.to_account_info(),
            &self.lp_token_program
        )
    }

    #[inline(never)]
    fn get_initial_locked_liquidity_mint_instruction(&self, initial_locked_liquidity: u64) -> MintTokensInstructions<'_, '_, '_, 'info>{
        MintTokensInstructions::new(
            initial_locked_liquidity,
            &self.lp_mint,
            self.cp_amm.to_account_info(),
            self.cp_amm_locked_lp_vault.to_account_info(),
            &self.lp_token_program
        )
    }
}