use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct LaunchCpAmm<'info>{
    #[account(mut)]
    signer: Signer<'info>,
    
}