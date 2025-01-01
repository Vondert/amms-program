use anchor_lang::context::CpiContext;
use anchor_lang::prelude::*;
use anchor_lang::ToAccountInfo;
use anchor_spl::token::{Mint, Token, TokenAccount, Burn, burn};
use crate::error::ErrorCode;

pub struct BurnTokensInstructions<'at, 'bt, 'ct, 'info> {
    amount: u64,
    cpi_context: CpiContext<'at, 'bt, 'ct, 'info, Burn<'info>>,
}

impl<'at, 'bt, 'ct, 'info> BurnTokensInstructions<'at, 'bt, 'ct, 'info> {
    pub fn new(amount: u64, mint: &Account<'info, Mint>, from: &Account<'info, TokenAccount>, from_authority: AccountInfo<'info>, token_program: &Program<'info, Token>) -> Result<Self>{
        require!(mint.supply.checked_sub(amount).is_some(), ErrorCode::LiquidityBurnOverflow);

        let cpi_context = CpiContext::new(
            token_program.to_account_info(),
            Burn{
                mint: mint.to_account_info(),
                from: from.to_account_info(),
                authority: from_authority,
            }
        );
        Ok(BurnTokensInstructions{
            amount,
            cpi_context,
        })
    }
    pub fn execute(mut self, optional_signers_seeds: Option<&'at[&'bt[&'ct[u8]]]>) -> Result<()>{
        if let Some(signer_seeds) = optional_signers_seeds {
            self.cpi_context = self.cpi_context.with_signer(signer_seeds);
        }
        burn(self.cpi_context, self.amount)
    }
}