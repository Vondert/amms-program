use anchor_lang::context::CpiContext;
use anchor_lang::prelude::{AccountInfo, InterfaceAccount, Program};
use anchor_lang::ToAccountInfo;
use anchor_spl::token::Token;
use anchor_spl::token_2022::{Token2022, TransferChecked};
use anchor_spl::token_interface::{Mint, TokenAccount};

pub struct TransferContextRegular<'at, 'bt, 'ct, 'info> {
    pub cpi_context: CpiContext<'at, 'bt, 'ct, 'info, TransferChecked<'info>>,
}


impl<'at, 'bt, 'ct, 'info>  TransferContextRegular<'at, 'bt, 'ct, 'info>  {
    pub(super) fn with_signers(self, signers_seeds: &'at[&'bt[&'ct[u8]]]) -> Self{
        Self {
            cpi_context: self.cpi_context.with_signer(signers_seeds),
        }
    }
    pub(super) fn new_for_spl_token(
        mint: &InterfaceAccount<'info, Mint>,
        from: &InterfaceAccount<'info, TokenAccount>,
        from_authority: AccountInfo<'info>,
        to: &InterfaceAccount<'info, TokenAccount>,
        token_program: &Program<'info, Token>
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
        mint: &InterfaceAccount<'info, Mint>,
        from: &InterfaceAccount<'info, TokenAccount>,
        from_authority: AccountInfo<'info>,
        to: &InterfaceAccount<'info, TokenAccount>,
        token_2022_program: &Program<'info, Token2022>
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