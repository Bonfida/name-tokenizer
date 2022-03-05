//! Create the NFT mint

use crate::{cpi::Cpi, state::MINT_PREFIX};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_program, sysvar,
    },
    spl_token::{instruction::initialize_mint, state::Mint},
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The mint of the NFT
    #[cons(writable)]
    pub mint: &'a T,

    /// The domain name account
    #[cons(writable)]
    pub name_account: &'a T,

    /// The central state account
    pub central_state: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// Rent sysvar account
    pub rent_account: &'a T,

    /// Fee payer account
    pub fee_payer: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            mint: next_account_info(accounts_iter)?,
            name_account: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            rent_account: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.rent_account, &sysvar::rent::ID)?;

        // Check owners
        check_account_owner(accounts.mint, &system_program::ID)?;
        check_account_owner(accounts.name_account, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let (mint, mint_nonce) = Pubkey::find_program_address(
        &[MINT_PREFIX, &accounts.name_account.key.to_bytes()],
        program_id,
    );
    check_account_key(accounts.mint, &mint)?;

    msg!("+ Creating mint");

    // Create mint account
    let seeds: &[&[u8]] = &[
        MINT_PREFIX,
        &accounts.name_account.key.to_bytes(),
        &[mint_nonce],
    ];
    Cpi::create_account(
        &spl_token::ID,
        accounts.system_program,
        accounts.fee_payer,
        accounts.mint,
        seeds,
        Mint::LEN,
    )?;

    // Initialize mint
    let ix = initialize_mint(
        &spl_token::ID,
        &mint,
        &crate::central_state::KEY,
        Some(&crate::central_state::KEY),
        0,
    )?;
    invoke_signed(
        &ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.mint.clone(),
            accounts.rent_account.clone(),
        ],
        &[seeds],
    )?;

    Ok(())
}
