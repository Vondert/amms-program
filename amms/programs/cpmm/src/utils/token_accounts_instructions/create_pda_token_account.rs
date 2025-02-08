use anchor_lang::prelude::*;
use anchor_lang::system_program::{create_account, CreateAccount};
use anchor_spl::{
    token::{ID as TOKEN_PROGRAM_ID, TokenAccount},
    token_2022::{
        ID as TOKEN_2022_PROGRAM_ID, 
        spl_token_2022::{
            state::{Mint, Account as Token22Account},
            extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions}
        }
    },
    token_interface::{InitializeAccount3, initialize_account3}
};
use crate::error::ErrorCode;


/// Represents an instruction to create and initialize a PDA token account.
///
/// This struct handles the creation of token accounts for both standard SPL tokens and SPL Token 2022 tokens.
///
/// # Fields
/// - `create_account_cpi_context`: Context for creating the token account using the system program.
/// - `initialize_cpi_context`: Context for initializing the token account with the appropriate token program.
/// - `lamports`: The minimum balance required for rent exemption.
/// - `space`: The amount of space to allocate for the token account.
/// - `token_program`: The public key of the token program (SPL Token or Token 2022).
pub(crate) struct CreatePdaTokenAccountInstruction<'at, 'bt, 'ct, 'info> {
    create_account_cpi_context: CpiContext<'at, 'bt, 'ct, 'info, CreateAccount<'info>>,
    initialize_cpi_context: CpiContext<'at, 'bt, 'ct, 'info, InitializeAccount3<'info>>,
    lamports: u64,
    space: u64,
    token_program: Pubkey
}
impl<'at, 'bt, 'ct, 'info> CreatePdaTokenAccountInstruction<'at, 'bt, 'ct, 'info>{

    /// Creates a new instance of `CreatePdaTokenAccountInstruction`.
    ///
    /// Determines the appropriate space allocation based on whether the token mint is a standard SPL token 
    /// or an SPL Token 2022 with extensions. It also calculates the minimum balance required for rent exemption.
    ///
    /// # Arguments
    /// - `signer`: The account paying for the creation of the token account.
    /// - `token_account`: The PDA token account to be created.
    /// - `authority`: The authority that will control the token account.
    /// - `mint`: The mint account of the token.
    /// - `token_program`: The token program (SPL Token or Token 2022).
    /// - `system_program`: The system program for creating accounts.
    ///
    /// # Errors
    /// Returns `ErrorCode::UnsupportedMint` if the mint is not supported.
    pub(crate) fn try_new(
        signer: AccountInfo<'info>,
        token_account: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        system_program: AccountInfo<'info>
    ) -> Result<Self>{

        // Determine the space required for the token account based on the mint type
        let space = match mint.owner{
            &TOKEN_PROGRAM_ID => TokenAccount::LEN,
            &TOKEN_2022_PROGRAM_ID => {
                let mint_data = mint.try_borrow_data()?;
                let mint_state = StateWithExtensions::<Mint>::unpack(&mint_data)?;
                let mint_extensions = mint_state.get_extension_types()?;
                let required_extensions = ExtensionType::get_required_init_account_extensions(&mint_extensions);
                ExtensionType::try_calculate_account_len::<Token22Account>(&required_extensions)?
            },
            _ => return Err(ErrorCode::UnsupportedMint.into())
        };
        let lamports = Rent::get()?.minimum_balance(space);
        
        let create_account_cpi_context = CpiContext::new(
            system_program,
            CreateAccount{
                from: signer,
                to: token_account.clone(),
            }
        );
        let initialize_cpi_context = CpiContext::new(
            token_program.clone(),
            InitializeAccount3 {
                account: token_account,
                mint,
                authority
            }
        );
        Ok(Self{
            create_account_cpi_context,
            initialize_cpi_context,
            token_program: token_program.key(),
            space: space as u64,
            lamports
        })
    }

    /// Executes the creation and initialization of the PDA token account.
    ///
    /// This method first creates the account using the system program and then initializes it
    /// using the appropriate token program (SPL Token or Token 2022).
    ///
    /// # Arguments
    /// - `signers_seeds`: The seeds required for signing the transaction if the account is a PDA.
    #[inline(never)]
    pub(crate) fn execute(self, signers_seeds: &'at[&'bt[&'ct[u8]]]) -> Result<()> {
        create_account(self.create_account_cpi_context.with_signer(signers_seeds), self.lamports, self.space, &self.token_program)?;
        initialize_account3(self.initialize_cpi_context)
    }
}