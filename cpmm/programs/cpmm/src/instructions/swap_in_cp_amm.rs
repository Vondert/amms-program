use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use crate::state::{AmmsConfig, cp_amm::CpAmm};
use crate::utils::token_instructions::{TransferTokensInstruction};

#[derive(Accounts)]
pub struct SwapInCpAmm<'info>{
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
    )]
    pub signer_base_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = quote_mint,
        associated_token::authority = signer,
    )]
    pub signer_quote_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [AmmsConfig::SEED, amms_config.id.to_le_bytes().as_ref()],
        bump = amms_config.bump
    )]
    pub amms_config: Box<Account<'info, AmmsConfig>>,

    #[account(
        mut,
        constraint = cp_amm.is_launched(),
        constraint = amms_config.key() == cp_amm.amms_config().key(),
        constraint = base_mint.key() == cp_amm.base_mint().key(),
        constraint = quote_mint.key() == cp_amm.quote_mint().key(),
        constraint = cp_amm_base_vault.key() == cp_amm.base_vault().key(),
        constraint = cp_amm_quote_vault.key() == cp_amm.quote_vault().key(),
        seeds = [CpAmm::SEED, lp_mint.key().as_ref()],
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
impl<'info> SwapInCpAmm<'info>{
    fn get_in_transfer_instruction(&self, in_amount: u64, is_in_out: bool) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        if is_in_out{
            TransferTokensInstruction::new(
                in_amount,
                &self.base_mint,
                &self.signer_base_account,
                self.signer.to_account_info(),
                &self.cp_amm_base_vault,
                &self.token_program,
                &self.token_2022_program
            )
        }
        else{
            TransferTokensInstruction::new(
                in_amount,
                &self.quote_mint,
                &self.signer_quote_account,
                self.signer.to_account_info(),
                &self.cp_amm_quote_vault,
                &self.token_program,
                &self.token_2022_program
            )
        }
    }
    fn get_out_transfer_instruction(&self, in_amount: u64, is_in_out: bool) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        if is_in_out{
            TransferTokensInstruction::new(
                in_amount,
                &self.quote_mint,
                &self.cp_amm_quote_vault,
                self.cp_amm.to_account_info(),
                &self.signer_quote_account,
                &self.token_program,
                &self.token_2022_program
            )

        }
        else{
            TransferTokensInstruction::new(
                in_amount,
                &self.base_mint,
                &self.cp_amm_base_vault,
                self.cp_amm.to_account_info(),
                &self.signer_base_account,
                &self.token_program,
                &self.token_2022_program
            )
        }
    }
}

pub(crate) fn handler(ctx: Context<SwapInCpAmm>, swap_amount: u64, estimated_result: u64, allowed_slippage: u64, is_in_out: bool) -> Result<()> {
    let in_transfer_instruction = ctx.accounts.get_in_transfer_instruction(swap_amount, is_in_out)?;
    let swap_payload = if is_in_out{
        ctx.accounts.cp_amm.get_base_to_quote_swap_payload(in_transfer_instruction.get_amount_after_fee(), estimated_result, allowed_slippage)?
    } else{
        ctx.accounts.cp_amm.get_quote_to_base_swap_payload(in_transfer_instruction.get_amount_after_fee(), estimated_result, allowed_slippage)?
    };
    let out_transfer_instruction = ctx.accounts.get_out_transfer_instruction(swap_payload.amount_to_withdraw(), is_in_out)?;
    in_transfer_instruction.execute(None)?;
    let cp_amm_seeds = ctx.accounts.cp_amm.seeds();
    let out_instruction_seeds: &[&[&[u8]]] = &[&cp_amm_seeds];
    out_transfer_instruction.execute(Some(&out_instruction_seeds))?;
    
    ctx.accounts.cp_amm.swap(swap_payload);
    
    Ok(())
}