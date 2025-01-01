use anchor_lang::context::CpiContext;
use anchor_lang::prelude::{AccountInfo, InterfaceAccount, Program};
use anchor_lang::ToAccountInfo;
use anchor_spl::token_2022::{Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};
use anchor_spl::token_2022_extensions::TransferCheckedWithFee;

pub(super) struct TransferContextWithFee<'at, 'bt, 'ct, 'info>  {
    pub fee: u64,
    pub cpi_context: CpiContext<'at, 'bt, 'ct, 'info, TransferCheckedWithFee<'info>>,
}
impl<'at, 'bt, 'ct, 'info>  TransferContextWithFee<'at, 'bt, 'ct, 'info>  {
    pub(super) fn with_signers(self, signers_seeds: &'at[&'bt[&'ct[u8]]]) -> Self{
        Self {
            fee: self.fee,
            cpi_context: self.cpi_context.with_signer(signers_seeds),
        }
    }
    pub(super) fn new_for_token_2022(
        fee: u64,
        mint: &InterfaceAccount<'info, Mint>,
        from: &InterfaceAccount<'info, TokenAccount>,
        from_authority: AccountInfo<'info>,
        to: &InterfaceAccount<'info, TokenAccount>,
        token_2022_program: &Program<'info, Token2022>
    ) -> Self{
        let cpi_context = CpiContext::new(
            token_2022_program.to_account_info(),
            TransferCheckedWithFee {
                token_program_id: token_2022_program.to_account_info(),
                source: from.to_account_info(),
                mint: mint.to_account_info(),
                destination: to.to_account_info(),
                authority: from_authority,
            }
        );
        Self{
            fee,
            cpi_context
        }
    }
}