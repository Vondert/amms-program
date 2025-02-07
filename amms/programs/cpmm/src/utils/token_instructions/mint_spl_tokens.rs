use anchor_lang::context::CpiContext;
use anchor_lang::prelude::*;
use anchor_lang::ToAccountInfo;
use anchor_spl::token::{mint_to, MintTo, Mint, Token};


/// Represents an instruction to mint tokens to a specified token account.
///
/// This struct prepares and executes the mint operation by encapsulating
/// the CPI (Cross-Program Invocation) context and the necessary parameters.
///
/// - `amount`: The amount of tokens to mint.
/// - `cpi_context`: The CPI context required to perform the mint operation.
pub(crate) struct MintTokensInstructions<'at, 'bt, 'ct, 'info> {
    amount: u64,
    cpi_context: CpiContext<'at, 'bt, 'ct, 'info, MintTo<'info>>,
}

impl<'at, 'bt, 'ct, 'info> MintTokensInstructions<'at, 'bt, 'ct, 'info> {
    /// Creates a new `MintTokensInstructions` instance for minting tokens.
    ///
    /// - `amount`: The amount of tokens to mint.
    /// - `mint`: The mint account of the token.
    /// - `mint_authority`: The authority allowed to mint tokens.
    /// - `to`: The destination token account where tokens will be minted.
    /// - `token_program`: The SPL token program responsible for handling the mint operation.
    pub fn new(amount: u64, mint: &Account<'info, Mint>, mint_authority: AccountInfo<'info>, to: AccountInfo<'info>, token_program: &Program<'info, Token>) -> Self{
        let cpi_context = CpiContext::new(
            token_program.to_account_info(), 
            MintTo{
                mint: mint.to_account_info(),
                to,
                authority: mint_authority,
            }
        );
        MintTokensInstructions{
            amount,
            cpi_context,
        }
    }

    /// Executes the mint operation.
    ///
    /// If signer seeds are provided, they are added to the CPI context to support PDA-based signing.
    ///
    /// - `optional_signers_seeds`: Optional signer seeds for PDA accounts.
    ///
    /// Returns:
    /// - `Ok(())` if the mint operation is successful.
    /// - `Err(ProgramError)` if the mint operation fails.
    pub fn execute(mut self, optional_signers_seeds: Option<&'at[&'bt[&'ct[u8]]]>) -> Result<()>{
        if let Some(signer_seeds) = optional_signers_seeds {
            self.cpi_context = self.cpi_context.with_signer(signer_seeds);
        }
        mint_to(self.cpi_context, self.amount)
    }
}