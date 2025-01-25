use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use crate::state::AmmsConfig;
use crate::state::cp_amm::CpAmm;
use crate::utils::token_instructions::TransferTokensInstruction;

#[derive(Accounts)]
pub struct CollectFeesFromCpAmm<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: Amms config's fee authority can be arbitrary
    pub fee_authority: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = base_mint,
        associated_token::authority = fee_authority,
    )]
    pub fee_authority_base_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = quote_mint,
        associated_token::authority = fee_authority,
    )]
    pub fee_authority_quote_account: Box<InterfaceAccount<'info, TokenAccount>>,
    
    #[account(
        constraint = amms_config.fee_authority.key() == fee_authority.key(),
        seeds = [AmmsConfig::SEED, amms_config.id.to_le_bytes().as_ref()],
        bump = amms_config.bump
    )]
    pub amms_config: Account<'info, AmmsConfig>,

    #[account(
        mut,
        constraint = cp_amm.is_launched(),
        constraint = amms_config.key() == cp_amm.amms_config().key(),
        constraint = base_mint.key() == cp_amm.base_mint().key(),
        constraint = quote_mint.key() == cp_amm.quote_mint().key(),
        constraint = cp_amm_base_vault.key() == cp_amm.base_vault().key(),
        constraint = cp_amm_quote_vault.key() == cp_amm.quote_vault().key(),
        seeds = [CpAmm::SEED, cp_amm.key().as_ref()],
        bump = cp_amm.bump()
    )]
    pub cp_amm: Box<Account<'info, CpAmm>>,

    #[account(mut)]
    pub cp_amm_base_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub cp_amm_quote_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> CollectFeesFromCpAmm<'info> {
    fn get_collect_base_fees_transfer_instruction(&self, base_fees: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::new(
            base_fees,
            &self.base_mint,
            &self.cp_amm_base_vault,
            self.cp_amm.to_account_info(),
            &self.fee_authority_base_account,
            &self.token_program,
            &self.token_2022_program
        )
    }
    fn get_collect_quote_fees_transfer_instruction(&self, quote_fees: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::new(
            quote_fees,
            &self.quote_mint,
            &self.fee_authority_quote_account,
            self.cp_amm.to_account_info(),
            &self.cp_amm_quote_vault,
            &self.token_program,
            &self.token_2022_program
        )
    }
}

pub(crate) fn handler(ctx: Context<CollectFeesFromCpAmm>) -> Result<()> {
    let collect_fees_payload = ctx.accounts.cp_amm.get_collect_fees_payload()?;
    let (protocol_base_fees_to_redeem, protocol_quote_fees_to_redeem) = (collect_fees_payload.protocol_base_fees_to_redeem(), collect_fees_payload.protocol_quote_fees_to_redeem());
    
    let cp_amm_seeds = ctx.accounts.cp_amm.seeds();
    let collect_fees_instruction_seeds: &[&[&[u8]]] = &[&cp_amm_seeds];
    
    if protocol_base_fees_to_redeem > 0{
        let collect_base_fees_instruction = Box::new(ctx.accounts.get_collect_base_fees_transfer_instruction(protocol_base_fees_to_redeem)?);
        collect_base_fees_instruction.execute(Some(collect_fees_instruction_seeds))?;
    }
    if protocol_quote_fees_to_redeem > 0{
        let collect_quote_fees_instruction = Box::new(ctx.accounts.get_collect_quote_fees_transfer_instruction(protocol_quote_fees_to_redeem)?);
        collect_quote_fees_instruction.execute(Some(collect_fees_instruction_seeds))?;
    }
    
    ctx.accounts.cp_amm.collect_fees(collect_fees_payload);
    Ok(())
}