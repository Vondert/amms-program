use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_2022::{Token2022};
use anchor_spl::token_interface::{get_mint_extension_data, transfer_checked, transfer_checked_with_fee, Mint, TokenAccount};
use anchor_spl::token_interface::spl_token_2022::extension::transfer_fee::TransferFeeConfig;
use crate::utils::transfer::{TransferContextRegular, TransferContextWithFee};
use crate::error::ErrorCode;

pub struct TokenTransferInstruction<'at, 'bt, 'ct, 'it> {
    amount: u64,
    decimals: u8,
    context: TransferContextType<'at, 'bt, 'ct, 'it>,
}
pub enum TransferContextType<'at, 'bt, 'ct, 'it> {
    Regular(TransferContextRegular<'at, 'bt, 'ct, 'it> ),
    WithFee(TransferContextWithFee<'at, 'bt, 'ct, 'it> )
}
impl<'at, 'bt, 'ct, 'it> TransferContextType<'at, 'bt, 'ct, 'it>{
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
impl<'at, 'bt, 'ct, 'it>  TokenTransferInstruction<'at, 'bt, 'ct, 'it>  {

    pub fn new(
        amount: u64, 
        mint: &InterfaceAccount<'it, Mint>, 
        from: &InterfaceAccount<'it, TokenAccount>, 
        from_authority: AccountInfo<'it>, 
        to: &InterfaceAccount<'it, TokenAccount>, 
        token_program: &Program<'it, Token>, 
        token_2022_program: &Program<'it, Token2022>,
        optional_signers_seeds: Option<&'at[&'bt[&'ct[u8]]]>
    ) -> Result<Self> {
        let mut context = if mint.to_account_info().owner.key() == token_program.key(){
            TransferContextType::Regular(
                TransferContextRegular::new_for_spl_token(
                    mint, from, from_authority, to, token_program
                )
            )
        }else if let Ok(transfer_fee_config) = get_mint_extension_data::<TransferFeeConfig>(&mint.to_account_info()){
            let fee = transfer_fee_config.calculate_epoch_fee(Clock::get()?.epoch, amount).ok_or(ErrorCode::MintTransferFeeCalculationFailed)?;
            TransferContextType::WithFee(
                TransferContextWithFee::new_for_token_2022(
                    fee, mint, from, from_authority, to, token_2022_program
                )
            )
        }else{
            TransferContextType::Regular(
                TransferContextRegular::new_for_token_2022(
                    mint, from, from_authority, to, token_2022_program
                )
            )
        };
        if let Some(signer_seeds) = optional_signers_seeds {
            context = context.add_signers_seeds(signer_seeds);
        }
        Ok(Self {
            amount,
            decimals: mint.decimals,
            context,
        })
    }
    pub fn execute_transfer(self) -> Result<()>{
        match self.context {
            TransferContextType::Regular(context) => {
                transfer_checked(context.cpi_context, self.amount, self.decimals)
            },
            TransferContextType::WithFee(context) => {
                transfer_checked_with_fee(context.cpi_context, self.amount, self.decimals, context.fee)
            }
        }
    }

}

