use anchor_lang::prelude::*;
use anchor_spl::{
    token::{ID as TOKEN_PROGRAM_ID}
};
use anchor_spl::token_interface::{get_mint_extension_data, transfer_checked, transfer_checked_with_fee, Mint, TokenInterface, TokenAccount};
use anchor_spl::token_interface::spl_token_2022::extension::transfer_fee::TransferFeeConfig;
use crate::utils::token_instructions::{TransferContextRegular, TransferContextWithFee};
use crate::error::ErrorCode;

/// Represents an instruction to transfer tokens between accounts.
///
/// This struct handles both standard SPL tokens and SPL Token 2022 with transfer fees.
///
/// # Fields
/// - `amount`: The amount of tokens to transfer.
/// - `decimals`: Number of decimal places in the token's representation.
/// - `context`: Encapsulates the transfer context, which can be either a regular transfer or one with fees.
pub(crate) struct TransferTokensInstruction<'at, 'bt, 'ct, 'info> {
    amount: u64,
    decimals: u8,
    context: TransferContextType<'at, 'bt, 'ct, 'info>,
}
impl<'at, 'bt, 'ct, 'info>  TransferTokensInstruction<'at, 'bt, 'ct, 'info>  {

    /// Creates a new instance of `TransferTokensInstruction`.
    ///
    /// Automatically determines whether the mint and token program require a regular transfer
    /// or one with transfer fees based on the token type and its associated metadata.
    ///
    /// - `amount`: The amount of tokens to transfer.
    /// - `mint`: The mint account of the token.
    /// - `from`: The source token account.
    /// - `from_authority`: Authority of the source account.
    /// - `to`: The destination token account.
    /// - `token_program`: Program for standard SPL tokens.
    /// - `token_2022_program`: Program for SPL Token 2022.
    pub fn try_new(
        amount: u64, 
        mint: &'_ InterfaceAccount<'info, Mint>, 
        from: &'_ InterfaceAccount<'info, TokenAccount>, 
        from_authority: AccountInfo<'info>,
        to: &'_ InterfaceAccount<'info, TokenAccount>, 
        token_program: &'_ Interface<'info, TokenInterface>
    ) -> Result<Self> {
        require!(from.amount >= amount, ErrorCode::InsufficientBalanceForTransfer);
        require!(mint.to_account_info().owner.key() == token_program.key(), ErrorCode::MintAndTokenProgramMismatch);
        
        let from_account_info = from.to_account_info();
        let to_account_info = to.to_account_info();
        
        let context = if mint.to_account_info().owner.key() == TOKEN_PROGRAM_ID {
            TransferContextType::Regular(
                TransferContextRegular::new_for_spl_token(
                    mint, from_account_info, from_authority, to_account_info, token_program
                )
            )
        }else if let Ok(transfer_fee_config) = get_mint_extension_data::<TransferFeeConfig>(&mint.to_account_info()){
            let fee = transfer_fee_config.calculate_epoch_fee(Clock::get()?.epoch, amount).ok_or(ErrorCode::MintTransferFeeCalculationFailed)?;
            TransferContextType::WithFee(
                TransferContextWithFee::new_for_token_2022(
                    fee, mint, from_account_info, from_authority, to_account_info, token_program
                )
            )
        }else{
            TransferContextType::Regular(
                TransferContextRegular::new_for_token_2022(
                    mint, from_account_info, from_authority, to_account_info, token_program
                )
            )
        };

        Ok(Self {
            amount,
            decimals: mint.decimals,
            context,
        })
    }

    /// Executes the transfer operation.
    ///
    /// - `optional_signers_seeds`: Optional signer seeds for PDA accounts.
    ///
    /// Returns:
    /// - `Ok(())` if the transfer is successful.
    /// - `Err(ErrorCode)` if the transfer fails.
    #[inline(never)]
    pub fn execute(mut self, optional_signers_seeds: Option<&'at[&'bt[&'ct[u8]]]>) -> Result<()>{
        if let Some(signer_seeds) = optional_signers_seeds {
            self.context = self.context.add_signers_seeds(signer_seeds);
        }
        match self.context {
            TransferContextType::Regular(context) => {
                transfer_checked(context.cpi_context, self.amount, self.decimals)
            },
            TransferContextType::WithFee(context) => {
                transfer_checked_with_fee(context.cpi_context, self.amount, self.decimals, context.fee)
            }
        }
    }

    /// Gets the number of decimals for the token.
    ///
    /// Returns:
    /// - The number of decimals.
    #[inline]
    pub fn get_decimals(&self) -> u8{
        self.decimals
    }
    
    /// Calculates the amount of tokens that will be received after deducting transfer fees.
    ///
    /// Returns:
    /// - The net amount of tokens after fees.
    #[inline]
    pub fn get_amount_after_fee(&self) -> u64{
        self.get_raw_amount().checked_sub(self.get_fee()).unwrap()
    }

    /// Retrieves the transfer fee for the transaction.
    ///
    /// Returns:
    /// - The fee amount for the transaction.
    #[inline]
    pub fn get_fee(&self) -> u64{
        match &self.context{
            TransferContextType::Regular(_) => {
                0
            },
            TransferContextType::WithFee(context) => {
                context.fee
            }
        }
    }

    /// Retrieves the raw amount of tokens to be transferred.
    ///
    /// Returns:
    /// - The raw transfer amount.
    #[inline]
    pub fn get_raw_amount(&self) -> u64{
        self.amount
    }
}

/// Represents the context of the transfer operation, which can be either:
/// - `Regular`: For transfers without fees.
/// - `WithFee`: For transfers that include a transfer fee.
enum TransferContextType<'at, 'bt, 'ct, 'info> {
    Regular(TransferContextRegular<'at, 'bt, 'ct, 'info> ),
    WithFee(TransferContextWithFee<'at, 'bt, 'ct, 'info> )
}
impl<'at, 'bt, 'ct> TransferContextType<'at, 'bt, 'ct, '_>{

    /// Adds signer seeds to the context for PDA account signing.
    ///
    /// - `signers_seeds`: The seeds for signing.
    ///
    /// Returns:
    /// - A new context with the signer seeds added.
    #[inline]
    fn add_signers_seeds(self, signers_seeds: &'at[&'bt[&'ct[u8]]]) -> Self {
        match self {
            TransferContextType::Regular(context) => {
                TransferContextType::Regular(context.with_signers(signers_seeds))
            },
            TransferContextType::WithFee(context) => {
                TransferContextType::WithFee(context.with_signers(signers_seeds))
            }
        }
    }
}