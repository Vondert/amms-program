use anchor_lang::context::CpiContext;
use anchor_lang::prelude::{AccountInfo, InterfaceAccount, Program};
use anchor_lang::ToAccountInfo;
use anchor_spl::token::Token;
use anchor_spl::token_2022::{Token2022, TransferChecked};
use anchor_spl::token_interface::{Mint, TokenAccount};

pub struct TransferContextRegular<'at, 'bt, 'ct, 'it> {
    pub cpi_context: CpiContext<'at, 'bt, 'ct, 'it, TransferChecked<'it>>,
}


impl<'at, 'bt, 'ct, 'it>  TransferContextRegular<'at, 'bt, 'ct, 'it>  {
    pub(super) fn with_signers(self, signers_seeds: &'at[&'bt[&'ct[u8]]]) -> Self{
        Self {
            cpi_context: self.cpi_context.with_signer(signers_seeds),
        }
    }
    pub(super) fn new_for_spl_token(
        mint: &InterfaceAccount<'it, Mint>,
        from: &InterfaceAccount<'it, TokenAccount>,
        from_authority: AccountInfo<'it>,
        to: &InterfaceAccount<'it, TokenAccount>,
        token_program: &Program<'it, Token>
    ) -> Self{
        let cpi_context = CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from: from.to_account_info(),
                mint: mint.to_account_info(),
                to: to.to_account_info(),
                authority: from_authority,
            }
        );
        Self{
            cpi_context
        }
    }
    pub(super) fn new_for_token_2022(
        mint: &InterfaceAccount<'it, Mint>,
        from: &InterfaceAccount<'it, TokenAccount>,
        from_authority: AccountInfo<'it>,
        to: &InterfaceAccount<'it, TokenAccount>,
        token_2022_program: &Program<'it, Token2022>
    ) -> Self{
        let cpi_context = CpiContext::new(
            token_2022_program.to_account_info(),
            TransferChecked {
                from: from.to_account_info(),
                mint: mint.to_account_info(),
                to: to.to_account_info(),
                authority: from_authority,
            }
        );
        Self{
            cpi_context
        }
    }
}