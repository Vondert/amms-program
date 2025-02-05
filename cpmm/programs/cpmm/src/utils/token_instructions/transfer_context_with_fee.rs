use anchor_lang::context::CpiContext;
use anchor_lang::prelude::{AccountInfo, Interface, InterfaceAccount};
use anchor_lang::ToAccountInfo;
use anchor_spl::token_interface::{Mint, TokenInterface};
use anchor_spl::token_2022_extensions::TransferCheckedWithFee;

/// Represents the context for a token transfer that includes a transfer fee.
///
/// This struct encapsulates the required data for executing SPL Token 2022
/// transfers that apply a fee to the transfer operation.
///
/// - `fee`: The fee amount to be applied to the transfer.
/// - `cpi_context`: The CPI context for performing the transfer with the fee.
pub(super) struct TransferContextWithFee<'at, 'bt, 'ct, 'info> {
    pub fee: u64,
    pub cpi_context: CpiContext<'at, 'bt, 'ct, 'info, TransferCheckedWithFee<'info>>,
}

impl<'at, 'bt, 'ct, 'info> TransferContextWithFee<'at, 'bt, 'ct, 'info> {
    /// Adds signer seeds to the transfer context for PDA-based account signing.
    ///
    /// - `signers_seeds`: The signer seeds for the account.
    ///
    /// Returns:
    /// - A new `TransferContextWithFee` instance with the signer seeds included.
    pub(super) fn with_signers(self, signers_seeds: &'at[&'bt[&'ct[u8]]]) -> Self {
        Self {
            fee: self.fee,
            cpi_context: self.cpi_context.with_signer(signers_seeds),
        }
    }

    /// Creates a new transfer context for an SPL Token 2022 transfer with a fee.
    ///
    /// - `fee`: The calculated fee for the transfer.
    /// - `mint`: The mint account of the token.
    /// - `from`: The source token account.
    /// - `from_authority`: The authority of the source account.
    /// - `to`: The destination token account.
    /// - `token_2022_program`: The program for SPL Token 2022.
    ///
    /// Returns:
    /// - A new `TransferContextWithFee` instance initialized for SPL Token 2022 transfers with a fee.
    pub(super) fn new_for_token_2022(
        fee: u64,
        mint: &InterfaceAccount<'info, Mint>,
        from: AccountInfo<'info>,
        from_authority: AccountInfo<'info>,
        to: AccountInfo<'info>,
        token_program: &Interface<'info, TokenInterface>,
    ) -> Self {
        let cpi_context = CpiContext::new(
            token_program.to_account_info(),
            TransferCheckedWithFee {
                token_program_id: token_program.to_account_info(),
                source: from,
                mint: mint.to_account_info(),
                destination: to,
                authority: from_authority,
            },
        );
        Self {
            fee,
            cpi_context,
        }
    }
}