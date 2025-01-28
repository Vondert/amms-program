use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use crate::error::ErrorCode;
pub(crate) struct TransferLamportsInstruction<'at, 'bt, 'ct, 'info>{
    lamports: u64,
    cpi_context: CpiContext<'at, 'bt, 'ct, 'info, Transfer<'info>>
}
impl<'at, 'bt, 'ct, 'info> TransferLamportsInstruction<'at, 'bt, 'ct, 'info>{
    pub fn new(lamports: u64, from: AccountInfo<'info>, to: AccountInfo<'info>, system_program: &Program<'info , System>) -> Result<Self> {
        require!(**from.lamports.borrow() >= lamports, ErrorCode::InsufficientBalanceForTransfer);
        let cpi_context = CpiContext::new(
            system_program.to_account_info(),
            Transfer{
                from,
                to,
            }
        );
        
        Ok(Self{
            lamports,
            cpi_context
        })
    }
    pub fn execute(self) -> Result<()> {
        transfer(self.cpi_context, self.lamports)
    }
}