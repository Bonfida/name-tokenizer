//! Create a verified MPL Core collection

use crate::{
    state::{COLLECTION_CORE_NAME, COLLECTION_CORE_URI, CORE_COLLECTION_PREFIX},
    utils::{get_core_collection_key, get_plugins},
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
    pub central_state: &'a T,

    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The metadata program account
    pub mpl_core_program: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// Collection account
    #[cons(writable)]
    pub collection: &'a T,
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
            mpl_core_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            collection: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;
        check_account_key(accounts.mpl_core_program, &mpl_core::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;

        // Check owners
        check_account_owner(accounts.collection, &system_program::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let (collection, collection_nonce) = get_core_collection_key();
    check_account_key(accounts.collection, &collection)?;

    mpl_core::instructions::CreateCollectionV1Cpi::new(
        accounts.mpl_core_program,
        mpl_core::instructions::CreateCollectionV1CpiAccounts {
            system_program: accounts.system_program,
            update_authority: Some(accounts.central_state),
            payer: accounts.fee_payer,
            collection: accounts.collection,
        },
        mpl_core::instructions::CreateCollectionV1InstructionArgs {
            name: COLLECTION_CORE_NAME.to_string(),
            uri: COLLECTION_CORE_URI.to_string(),
            plugins: Some(get_plugins()),
        },
    )
    .invoke_signed(&[&[
        CORE_COLLECTION_PREFIX,
        &program_id.to_bytes(),
        &[collection_nonce],
    ]])?;

    Ok(())
}
