use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, Token}, token_interface};
use anchor_spl::token_interface::TokenInterface;
use crate::constants::CP_AMM_INITIALIZE_PRICE_IN_LAMPORTS;
use crate::state::{AmmsConfig, cp_amm::{
    CpAmm, 
    CpAmmCalculate
}};
use crate::utils::system_instructions::TransferLamportsInstruction;
use crate::utils::token_accounts_instructions::CreatePdaTokenAccountInstruction;
use crate::utils::validate_tradable_mint;

#[derive(Accounts)]
pub struct InitializeCpAmm<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    /// CHECK: Amms config's fee authority can be arbitrary type
    pub fee_authority: AccountInfo<'info>,
    pub base_mint: Box<InterfaceAccount<'info, token_interface::Mint>>,
    #[account(
        constraint = base_mint.key() != quote_mint.key()
    )]
    pub quote_mint: Box<InterfaceAccount<'info, token_interface::Mint>>,
    
    #[account(
        init,
        payer = signer,
        mint::decimals = CpAmm::LP_MINT_INITIAL_DECIMALS,
        mint::authority = cp_amm,
        mint::token_program = lp_token_program
    )]
    pub lp_mint: Box<Account<'info, Mint>>,
    
    #[account(
        constraint = amms_config.fee_authority().key() == fee_authority.key(),
        seeds = [AmmsConfig::SEED, amms_config.id.to_le_bytes().as_ref()],
        bump = amms_config.bump()
    )]
    pub amms_config: Box<Account<'info, AmmsConfig>>,
    
    #[account(
        init,
        payer = signer,
        space = 8 + CpAmm::INIT_SPACE,
        seeds = [CpAmm::SEED, lp_mint.key().as_ref()],
        bump
    )]
    pub cp_amm: Box<Account<'info, CpAmm>>,

    #[account(
        mut,
        seeds = [CpAmm::VAULT_SEED, cp_amm.key().as_ref(), base_mint.key().as_ref()],
        bump
    )]
    pub cp_amm_base_vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [CpAmm::VAULT_SEED, cp_amm.key().as_ref(), quote_mint.key().as_ref()],
        bump
    )]
    pub cp_amm_quote_vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [CpAmm::VAULT_SEED, cp_amm.key().as_ref(), lp_mint.key().as_ref()],
        bump
    )]
    pub cp_amm_locked_lp_vault: AccountInfo<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub lp_token_program: Program<'info, Token>,
    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,
}

pub(crate) fn handler(ctx: Context<InitializeCpAmm>) -> Result<()> {
    ctx.accounts.validate_base_mint()?;
    ctx.accounts.validate_quote_mint()?;
    {
        let cp_amm_key = ctx.accounts.cp_amm.key();
        {
            let base_mint_key = ctx.accounts.base_mint.key();
            let create_cp_amm_base_vault = Box::new(ctx.accounts.get_create_cp_amm_base_vault_instruction()?);
            let cp_amm_base_vault_seeds = [CpAmm::VAULT_SEED, cp_amm_key.as_ref(), base_mint_key.as_ref(), &[ctx.bumps.cp_amm_base_vault]];
            create_cp_amm_base_vault.execute(&[&cp_amm_base_vault_seeds])?;
        }
        {
            let quote_mint_key = ctx.accounts.quote_mint.key();
            let create_cp_amm_quote_vault = Box::new(ctx.accounts.get_create_cp_amm_quote_vault_instruction()?);
            let cp_amm_quote_vault_seeds = [CpAmm::VAULT_SEED, cp_amm_key.as_ref(),quote_mint_key.as_ref(), &[ctx.bumps.cp_amm_quote_vault]];
            create_cp_amm_quote_vault.execute(&[&cp_amm_quote_vault_seeds])?;
        }
        {
            let lp_mint_key = ctx.accounts.lp_mint.key();
            let create_cp_amm_locked_lp_vault = Box::new(ctx.accounts.get_create_cp_amm_locked_lp_vault_instruction()?);
            let cp_amm_locked_lp_vault_seeds = [CpAmm::VAULT_SEED, cp_amm_key.as_ref(), lp_mint_key.as_ref(), &[ctx.bumps.cp_amm_locked_lp_vault]];
            create_cp_amm_locked_lp_vault.execute(&[&cp_amm_locked_lp_vault_seeds])?;
        }
    }
    let accounts = ctx.accounts;

    let pay_initial_lamports_instruction = Box::new(accounts.get_pay_initial_lamports_instruction(CP_AMM_INITIALIZE_PRICE_IN_LAMPORTS)?);
    pay_initial_lamports_instruction.execute()?;
    
    accounts.cp_amm.initialize(
        &accounts.base_mint,
        &accounts.quote_mint,
        &accounts.lp_mint,
        &accounts.amms_config,
        &accounts.signer.to_account_info(),
        &accounts.cp_amm_base_vault,
        &accounts.cp_amm_quote_vault,
        &accounts.cp_amm_locked_lp_vault,
        ctx.bumps.cp_amm,
        ctx.bumps.cp_amm_base_vault,
        ctx.bumps.cp_amm_quote_vault,
        ctx.bumps.cp_amm_locked_lp_vault
    )
}

impl<'info> InitializeCpAmm<'info>{
    fn validate_base_mint(&self) -> Result<()> {
        let base_mint = self.base_mint.as_ref();
        validate_tradable_mint(base_mint)
    }
    fn validate_quote_mint(&self) -> Result<()> {
        let quote_mint = self.quote_mint.as_ref();
        validate_tradable_mint(quote_mint)
    }
    fn get_pay_initial_lamports_instruction(&self, lamports: u64) -> Result<TransferLamportsInstruction<'_, '_, '_, 'info>>{
        TransferLamportsInstruction::new(
            lamports,
            self.signer.to_account_info(),
            self.fee_authority.to_account_info(),
            &self.system_program
        )
    }
    #[inline(never)]
    fn get_create_cp_amm_base_vault_instruction(&self) -> Result<CreatePdaTokenAccountInstruction<'_, '_, '_, 'info>>{
        CreatePdaTokenAccountInstruction::try_new(
            self.signer.to_account_info(),
            self.cp_amm_base_vault.to_account_info(),
            self.cp_amm.to_account_info(),
            self.base_mint.to_account_info(),
            self.base_token_program.to_account_info(),
            self.system_program.to_account_info()
        )
    }
    #[inline(never)]
    fn get_create_cp_amm_quote_vault_instruction(&self) -> Result<CreatePdaTokenAccountInstruction<'_, '_, '_, 'info>>{
        CreatePdaTokenAccountInstruction::try_new(
            self.signer.to_account_info(),
            self.cp_amm_quote_vault.to_account_info(),
            self.cp_amm.to_account_info(),
            self.quote_mint.to_account_info(),
            self.quote_token_program.to_account_info(),
            self.system_program.to_account_info()
        )
    }
    #[inline(never)]
    fn get_create_cp_amm_locked_lp_vault_instruction(&self) -> Result<CreatePdaTokenAccountInstruction<'_, '_, '_, 'info>>{
        CreatePdaTokenAccountInstruction::try_new(
            self.signer.to_account_info(),
            self.cp_amm_locked_lp_vault.to_account_info(),
            self.cp_amm.to_account_info(),
            self.lp_mint.to_account_info(),
            self.lp_token_program.to_account_info(),
            self.system_program.to_account_info()
        )
    }
}