use anchor_lang::prelude::*;
use anchor_spl::associated_token::{create, AssociatedToken, Create};
pub(crate) struct CreateAtaInstruction<'at, 'bt, 'ct, 'info> {
    cpi_context: CpiContext<'at, 'bt, 'ct, 'info, Create<'info>>
}
impl<'at, 'bt, 'ct, 'info> CreateAtaInstruction<'at, 'bt, 'ct, 'info>{
    pub(crate) fn new(
        signer: &Signer<'info>,
        token_account: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        associated_token_program: &Program<'info, AssociatedToken>,
        system_program: &Program<'info, System>,
        token_program: AccountInfo<'info>
    ) -> Self{
        let cpi_context = CpiContext::new(
            associated_token_program.to_account_info(), 
            Create {
                payer: signer.to_account_info(),
                associated_token: token_account,
                authority,
                mint,
                system_program: system_program.to_account_info(),
                token_program,
            }
        );

        Self{
            cpi_context
        }
    }

    pub(crate) fn execute(self) -> Result<()> {
        create(self.cpi_context)
    }
}