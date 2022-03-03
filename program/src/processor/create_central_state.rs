//! Create central state

use crate::{
    cpi::Cpi,
    state::{CentralState, Tag},
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The fee payer
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The system program account
    pub system_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            central_state: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &system_program::ID)?;

        // Check owners
        check_account_owner(accounts.central_state, &system_program::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    // Check derivation
    let (derived_key, nonce) = CentralState::find_key(program_id);
    check_account_key(accounts.central_state, &derived_key)?;

    let central_state = CentralState {
        tag: Tag::CentralState,
    };

    Cpi::create_account(
        program_id,
        accounts.system_program,
        accounts.fee_payer,
        accounts.central_state,
        &[&program_id.to_bytes(), &[nonce]],
        central_state.borsh_len(),
    )?;

    central_state.save(&mut accounts.central_state.data.borrow_mut());

    Ok(())
}
