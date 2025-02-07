use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{Token, TokenAccount},
    token_interface::{TokenAccount as InterfaceTokenAccount, Mint, TokenInterface}
};
use anchor_spl::associated_token::{
    get_associated_token_address_with_program_id,
    AssociatedToken
};
use crate::state::{AmmsConfig, cp_amm::CpAmm};
use crate::utils::{
    token_accounts_instructions::{CreateAtaInstruction},
    token_instructions::{MintTokensInstructions, TransferTokensInstruction}
};
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct LaunchCpAmm<'info>{
    #[account(mut)]
    pub signer: Signer<'info>,
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub lp_mint: Box<Account<'info, token::Mint>>,
    #[account(mut)]
    // Token program will check mint and authority via token_instructions instruction
    pub signer_base_account: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,
    #[account(mut)]
    // Token program will check mint and authority via token_instructions instruction
    pub signer_quote_account: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,

    #[account(
        mut,
        constraint = signer_lp_account.owner == signer.key()
    )]
    pub signer_lp_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [AmmsConfig::SEED, amms_config.id.to_le_bytes().as_ref()],
        bump = amms_config.bump()
    )]
    pub amms_config: Box<Account<'info, AmmsConfig>>,

    #[account(
        mut,
        constraint = !cp_amm.is_launched(),
        constraint = signer.key() == cp_amm.creator().key(),
        constraint = amms_config.key() == cp_amm.amms_config().key(),
        constraint = lp_mint.key() == cp_amm.lp_mint,
        constraint = base_mint.key() == cp_amm.base_mint().key(),
        constraint = quote_mint.key() == cp_amm.quote_mint().key(),
        seeds = [CpAmm::SEED, cp_amm.lp_mint.as_ref()],
        bump = cp_amm.bump()
    )]
    pub cp_amm: Box<Account<'info, CpAmm>>,

    #[account(mut)]
    /// CHECK: This is the ATA for CpAmm to hold base token liquidity.
    ///        - **Validation**: Checked inside `check_or_initialize_cp_amms_vaults()`:
    ///          `get_associated_token_address_with_program_id(cp_amm, quote_mint, quote_token_program)`.
    ///        - **Initialization**: If empty (`data_is_empty()`), it is created using CreateAtaInstruction.
    pub cp_amm_base_vault: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: This is the ATA for CpAmm to hold quote token liquidity.
    ///        - **Validation**: Checked inside `check_or_initialize_cp_amms_vaults()`:
    ///          `get_associated_token_address_with_program_id(cp_amm, base_mint, base_token_program)`.
    ///        - **Initialization**: If empty (`data_is_empty()`), it is created using CreateAtaInstruction.
    pub cp_amm_quote_vault: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: This is the ATA for CpAmm to store initially locked LP tokens.
    ///        - **Validation**: Checked inside `check_or_initialize_cp_amms_vaults()`:
    ///          `get_associated_token_address_with_program_id(cp_amm, lp_mint, lp_token_program)`.
    ///        - **Initialization**: If empty (`data_is_empty()`), it is created using CreateAtaInstruction.
    pub cp_amm_locked_lp_vault: AccountInfo<'info>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub lp_token_program: Program<'info, Token>,
    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<LaunchCpAmm>, base_liquidity: u64, quote_liquidity: u64) -> Result<()> {
    ctx.accounts.check_or_initialize_cp_amms_vaults()?;
    
    let provide_base_liquidity_instruction = Box::new(ctx.accounts.get_provide_base_liquidity_transfer_instruction(base_liquidity)?);
    let provide_quote_liquidity_instruction = Box::new(ctx.accounts.get_provide_quote_liquidity_transfer_instruction(quote_liquidity)?);

    let base_liquidity_to_provide = provide_base_liquidity_instruction.get_amount_after_fee();
    let quote_liquidity_to_provide = provide_quote_liquidity_instruction.get_amount_after_fee();

    let launch_payload = Box::new(ctx.accounts.cp_amm.get_launch_payload(base_liquidity_to_provide, quote_liquidity_to_provide)?);

    let launch_liquidity_mint_instruction = Box::new(ctx.accounts.get_launch_liquidity_mint_instruction(launch_payload.launch_liquidity()));
    let initial_locked_liquidity_mint_instruction = Box::new(ctx.accounts.get_initial_locked_liquidity_mint_instruction(launch_payload.initial_locked_liquidity()));

    provide_base_liquidity_instruction.execute(None)?;
    provide_quote_liquidity_instruction.execute(None)?;

    let cp_amm_seeds = ctx.accounts.cp_amm.seeds();
    let mint_instruction_seeds: &[&[&[u8]]] = &[&cp_amm_seeds];

    launch_liquidity_mint_instruction.execute(Some(mint_instruction_seeds))?;
    initial_locked_liquidity_mint_instruction.execute(Some(mint_instruction_seeds))?;

    ctx.accounts.cp_amm.launch(*launch_payload, ctx.accounts.cp_amm_base_vault.key(), ctx.accounts.cp_amm_quote_vault.key(), ctx.accounts.cp_amm_locked_lp_vault.key());
    Ok(())
}

impl<'info> LaunchCpAmm<'info>{

    #[inline(never)]
    fn get_provide_base_liquidity_transfer_instruction(&self, base_liquidity: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::try_new(
            base_liquidity,
            &self.base_mint,
            self.signer_base_account.to_account_info(),
            self.signer.to_account_info(),
            self.cp_amm_base_vault.to_account_info(),
            &self.base_token_program
        )
    }

    #[inline(never)]
    fn get_provide_quote_liquidity_transfer_instruction(&self, quote_liquidity: u64) -> Result<TransferTokensInstruction<'_, '_, '_, 'info>>{
        TransferTokensInstruction::try_new(
            quote_liquidity,
            &self.quote_mint,
            self.signer_quote_account.to_account_info(),
            self.signer.to_account_info(),
            self.cp_amm_quote_vault.to_account_info(),
            &self.quote_token_program
        )
    }

    #[inline(never)]
    fn get_launch_liquidity_mint_instruction(&self, launch_liquidity: u64) -> MintTokensInstructions<'_, '_, '_, 'info>{
        MintTokensInstructions::new(
            launch_liquidity,
            &self.lp_mint,
            self.cp_amm.to_account_info(),
            self.signer_lp_account.to_account_info(),
            &self.lp_token_program
        )
    }

    #[inline(never)]
    fn get_initial_locked_liquidity_mint_instruction(&self, initial_locked_liquidity: u64) -> MintTokensInstructions<'_, '_, '_, 'info>{
        MintTokensInstructions::new(
            initial_locked_liquidity,
            &self.lp_mint,
            self.cp_amm.to_account_info(),
            self.cp_amm_locked_lp_vault.to_account_info(),
            &self.lp_token_program
        )
    }

    #[inline(never)]
    fn get_create_cp_amm_base_vault_instruction(&self) -> CreateAtaInstruction<'_, '_, '_, 'info>{

        CreateAtaInstruction::new(
            &self.signer,
            self.cp_amm_base_vault.to_account_info(),
            self.cp_amm.to_account_info(),
            self.base_mint.to_account_info(),
            &self.associated_token_program,
            &self.system_program,
            self.base_token_program.to_account_info()
        )
    }
    
    #[inline(never)]
    fn get_create_cp_amm_quote_vault_instruction(&self) -> CreateAtaInstruction<'_, '_, '_, 'info>{
        CreateAtaInstruction::new(
            &self.signer,
            self.cp_amm_quote_vault.to_account_info(),
            self.cp_amm.to_account_info(),
            self.quote_mint.to_account_info(),
            &self.associated_token_program,
            &self.system_program,
            self.quote_token_program.to_account_info()
        )
    }
    
    #[inline(never)]
    fn get_create_cp_amm_locked_lp_vault_instruction(&self) -> CreateAtaInstruction<'_, '_, '_, 'info>{
        CreateAtaInstruction::new(
            &self.signer,
            self.cp_amm_locked_lp_vault.to_account_info(),
            self.cp_amm.to_account_info(),
            self.lp_mint.to_account_info(),
            &self.associated_token_program,
            &self.system_program,
            self.lp_token_program.to_account_info()
        )
    }
    
    #[inline(never)]
    fn check_or_initialize_cp_amms_vaults(&self) -> Result<()>{
        let cp_amm = &self.cp_amm.key();
        require_keys_eq!(self.cp_amm_base_vault.key(),  get_associated_token_address_with_program_id(
            cp_amm,
            &self.base_mint.key(),
            &self.base_token_program.key()
        ), ErrorCode::InvalidCpAmmVaultAddress);
        require_keys_eq!(self.cp_amm_quote_vault.key(),  get_associated_token_address_with_program_id(
            cp_amm,
            &self.quote_mint.key(),
            &self.quote_token_program.key()
        ), ErrorCode::InvalidCpAmmVaultAddress);
        require_keys_eq!(self.cp_amm_locked_lp_vault.key(),  get_associated_token_address_with_program_id(
            cp_amm,
            &self.lp_mint.key(),
            &self.lp_token_program.key()
        ), ErrorCode::InvalidCpAmmVaultAddress);

        if self.cp_amm_base_vault.data_is_empty(){
            self.get_create_cp_amm_base_vault_instruction().execute()?;
        }
        {
            let cp_amm_base_vault = InterfaceTokenAccount::try_deserialize(&mut &self.cp_amm_base_vault.try_borrow_data()?[..])?;
            require_keys_eq!(cp_amm_base_vault.owner, cp_amm.key(), ErrorCode::InvalidCpAmmVaultOwner);
        }

        if self.cp_amm_quote_vault.data_is_empty(){
            self.get_create_cp_amm_quote_vault_instruction().execute()?;
        }
        {
            let cp_amm_quote_vault = InterfaceTokenAccount::try_deserialize(&mut &self.cp_amm_quote_vault.try_borrow_data()?[..])?;
            require_keys_eq!(cp_amm_quote_vault.owner, cp_amm.key(), ErrorCode::InvalidCpAmmVaultOwner);
        }
        
        if self.cp_amm_locked_lp_vault.data_is_empty(){
            self.get_create_cp_amm_locked_lp_vault_instruction().execute()?;
        }
        {
            let cp_amm_locked_lp_vault = InterfaceTokenAccount::try_deserialize(&mut &self.cp_amm_locked_lp_vault.try_borrow_data()?[..])?;
            require_keys_eq!(cp_amm_locked_lp_vault.owner, cp_amm.key(), ErrorCode::InvalidCpAmmVaultOwner);
        }
        
        Ok(())
    }
}

