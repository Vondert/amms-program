use anchor_lang::context::CpiContext;
use anchor_lang::prelude::{AccountInfo, Interface, InterfaceAccount};
use anchor_lang::ToAccountInfo;
use anchor_spl::token_2022::{TransferChecked};
use anchor_spl::token_interface::{Mint, TokenInterface};

/// Represents the context for a regular token transfer without fees.
///
/// This struct is used to encapsulate the required data for executing token
/// transfers, including the CPI context for either
/// standard SPL tokens or SPL Token 2022.
///
/// - `cpi_context`: The CPI context for performing the transfer.
pub(super) struct TransferContextRegular<'at, 'bt, 'ct, 'info> {
    pub cpi_context: CpiContext<'at, 'bt, 'ct, 'info, TransferChecked<'info>>,
}

impl<'at, 'bt, 'ct, 'info>  TransferContextRegular<'at, 'bt, 'ct, 'info>  {
    /// Adds signer seeds to the transfer context for PDA-based account signing.
    ///
    /// - `signers_seeds`: The signer seeds for the account.
    ///
    /// Returns:
    /// - A new `TransferContextRegular` instance with the signer seeds included.
    pub(super) fn with_signers(self, signers_seeds: &'at[&'bt[&'ct[u8]]]) -> Self{
        Self {
            cpi_context: self.cpi_context.with_signer(signers_seeds),
        }
    }

    /// Creates a new transfer context for a standard SPL token.
    ///
    /// - `mint`: The mint account of the token.
    /// - `from`: The source token account.
    /// - `from_authority`: The authority of the source account.
    /// - `to`: The destination token account.
    /// - `token_program`: The program for standard SPL tokens.
    ///
    /// Returns:
    /// - A new `TransferContextRegular` instance initialized for SPL token transfers.
    pub(super) fn new_for_spl_token(
        mint: &InterfaceAccount<'info, Mint>,
        from: AccountInfo<'info>,
        from_authority: AccountInfo<'info>,
        to: AccountInfo<'info>,
        token_program: &Interface<'info, TokenInterface>,
    ) -> Self{
        let cpi_context = CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from,
                mint: mint.to_account_info(),
                to,
                authority: from_authority,
            }
        );
        Self{
            cpi_context
        }
    }

    /// Creates a new transfer context for an SPL Token 2022.
    ///
    /// - `mint`: The mint account of the token.
    /// - `from`: The source token account.
    /// - `from_authority`: The authority of the source account.
    /// - `to`: The destination token account.
    /// - `token_2022_program`: The program for SPL Token 2022.
    ///
    /// Returns:
    /// - A new `TransferContextRegular` instance initialized for SPL Token 2022 transfers.
    pub(super) fn new_for_token_2022(
        mint: &InterfaceAccount<'info, Mint>,
        from: AccountInfo<'info>,
        from_authority: AccountInfo<'info>,
        to: AccountInfo<'info>,
        token_program: &Interface<'info, TokenInterface>,
    ) -> Self{
        let cpi_context = CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from,
                mint: mint.to_account_info(),
                to,
                authority: from_authority,
            }
        );
        Self{
            cpi_context
        }
    }
}