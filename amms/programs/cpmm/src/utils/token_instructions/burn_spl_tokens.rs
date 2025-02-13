use anchor_lang::context::CpiContext;
use anchor_lang::prelude::*;
use anchor_lang::ToAccountInfo;
use anchor_spl::token::{Mint, Token, Burn, burn, TokenAccount};
use crate::error::ErrorCode;

/// Represents an instruction to burn tokens from a token account.
///
/// This struct handles the process of burning tokens by preparing a CPI
/// context and executing the burn operation.
///
/// - `amount`: The amount of tokens to burn.
/// - `cpi_context`: The CPI context required to perform the burn operation.
pub(crate) struct BurnTokensInstructions<'at, 'bt, 'ct, 'info> {
    amount: u64,
    cpi_context: CpiContext<'at, 'bt, 'ct, 'info, Burn<'info>>,
}

impl<'at, 'bt, 'ct, 'info> BurnTokensInstructions<'at, 'bt, 'ct, 'info> {
    /// Creates a new `BurnTokensInstructions` instance for burning tokens.
    ///
    /// - `amount`: The amount of tokens to burn.
    /// - `mint`: The mint account of the token.
    /// - `from`: The token account from which tokens will be burned.
    /// - `from_authority`: The authority of the token account.
    /// - `token_program`: The SPL token program responsible for handling the burn operation.
    pub fn try_new(amount: u64, mint: &Account<'info, Mint>, from: &Account<'info, TokenAccount>, from_authority: AccountInfo<'info>, token_program: &Program<'info, Token>) -> Result<Self>{
        require!(from.amount >= amount, ErrorCode::InsufficientBalanceForTransfer);
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


    /// Executes the burn operation.
    ///
    /// If signer seeds are provided, they are added to the CPI context to support PDA-based signing.
    ///
    /// - `optional_signers_seeds`: Optional signer seeds for PDA accounts.
    ///
    /// Returns:
    /// - `Ok(())` if the burn operation is successful.
    /// - `Err(ProgramError)` if the burn operation fails.
    pub fn execute(mut self, optional_signers_seeds: Option<&'at[&'bt[&'ct[u8]]]>) -> Result<()>{
        if let Some(signer_seeds) = optional_signers_seeds {
            self.cpi_context = self.cpi_context.with_signer(signer_seeds);
        }
        burn(self.cpi_context, self.amount)
    }
}