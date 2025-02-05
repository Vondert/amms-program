use anchor_lang::prelude::*;
use anchor_spl::token_interface::{InitializeAccount3, initialize_account3};
pub(crate) struct InitializeTokenAccountInstruction<'at, 'bt, 'ct, 'info> {
    cpi_context: CpiContext<'at, 'bt, 'ct, 'info, InitializeAccount3<'info>>
}
impl<'at, 'bt, 'ct, 'info> InitializeTokenAccountInstruction<'at, 'bt, 'ct, 'info>{
    pub(crate) fn new(
        token_account: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        token_program: AccountInfo<'info>
    ) -> Self{
        let cpi_context = CpiContext::new(
            token_program.to_account_info(),
            InitializeAccount3 {
                account: token_account,
                mint,
                authority
            }
        );
        Self{
            cpi_context
        }
    }

    pub(crate) fn execute(self) -> Result<()> {
        initialize_account3(self.cpi_context)
    }
}