use anchor_lang::context::CpiContext;
use anchor_lang::prelude::*;
use anchor_lang::ToAccountInfo;
use anchor_spl::token::{mint_to, MintTo, Mint, Token, TokenAccount};
use crate::error::ErrorCode;
pub struct MintTokensInstructions<'at, 'bt, 'ct, 'info> {
    amount: u64,
    cpi_context: CpiContext<'at, 'bt, 'ct, 'info, MintTo<'info>>,
}

impl<'at, 'bt, 'ct, 'info> MintTokensInstructions<'at, 'bt, 'ct, 'info> {
    pub fn new(amount: u64, mint: &Account<'info, Mint>, mint_authority: AccountInfo<'info>, to: &Account<'info, TokenAccount>, token_program: &Program<'info, Token>) -> Result<Self>{
        require!(mint.supply.checked_add(amount).is_some(), ErrorCode::LiquidityMintOverflow);
        
        let cpi_context = CpiContext::new(
            token_program.to_account_info(), 
            MintTo{
                mint: mint.to_account_info(),
                to: to.to_account_info(),
                authority: mint_authority,
            }
        );
        Ok(MintTokensInstructions{
            amount,
            cpi_context,
        })
    }
    pub fn execute(mut self, optional_signers_seeds: Option<&'at[&'bt[&'ct[u8]]]>) -> Result<()>{
        if let Some(signer_seeds) = optional_signers_seeds {
            self.cpi_context = self.cpi_context.with_signer(signer_seeds);
        }
        mint_to(self.cpi_context, self.amount)
    }
}