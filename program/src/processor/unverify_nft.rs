//! Unverify an NFT

use mpl_token_metadata::{
    accounts::{MasterEdition, Metadata},
    instructions::{UnverifyCollectionCpi, UnverifyCollectionCpiAccounts},
};

use crate::state::{COLLECTION_PREFIX, METADATA_SIGNER};

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
        system_program, sysvar,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The metadata account
    #[cons(writable)]
    pub metadata_account: &'a T,

    /// Master edition account
    pub edition_account: &'a T,

    /// Collection
    pub collection_metadata: &'a T,

    /// Mint of the collection
    pub collection_mint: &'a T,

    /// The central state account
    #[cons(writable)]
    pub central_state: &'a T,

    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The metadata program account
    pub metadata_program: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// Rent sysvar account
    pub rent_account: &'a T,

    /// The metadata signer
    #[cons(signer)]
    #[cfg(not(feature = "devnet"))]
    pub metadata_signer: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            metadata_account: next_account_info(accounts_iter)?,
            edition_account: next_account_info(accounts_iter)?,
            collection_metadata: next_account_info(accounts_iter)?,
            collection_mint: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            metadata_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            rent_account: next_account_info(accounts_iter)?,
            #[cfg(not(feature = "devnet"))]
            metadata_signer: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;
        check_account_key(accounts.metadata_program, &mpl_token_metadata::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.rent_account, &sysvar::rent::ID)?;
        #[cfg(not(feature = "devnet"))]
        check_account_key(accounts.metadata_signer, &METADATA_SIGNER)?;

        // Check owners
        check_account_owner(accounts.metadata_account, &mpl_token_metadata::ID)?;
        check_account_owner(accounts.edition_account, &mpl_token_metadata::ID)?;
        check_account_owner(accounts.collection_metadata, &mpl_token_metadata::ID)?;
        check_account_owner(accounts.collection_mint, &spl_token::ID)?;

        #[cfg(not(feature = "devnet"))]
        check_signer(accounts.metadata_signer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;

    // Verify edition PDA
    let (collection_mint, _) =
        Pubkey::find_program_address(&[COLLECTION_PREFIX, &program_id.to_bytes()], program_id);
    check_account_key(accounts.collection_mint, &collection_mint)?;

    let (edition_key, _) = MasterEdition::find_pda(&collection_mint);
    check_account_key(accounts.edition_account, &edition_key)?;

    // Verify collection metadata PDA
    let (collection_metadata, _) = Metadata::find_pda(&collection_mint);
    check_account_key(accounts.collection_metadata, &collection_metadata)?;

    let seeds: &[&[u8]] = &[&program_id.to_bytes(), &[crate::central_state::NONCE]];

    UnverifyCollectionCpi::new(
        accounts.metadata_program,
        UnverifyCollectionCpiAccounts {
            metadata: accounts.metadata_account,
            collection_authority: accounts.central_state,
            collection_mint: accounts.collection_mint,
            collection: accounts.collection_metadata,
            collection_master_edition_account: accounts.edition_account,
            collection_authority_record: None,
        },
    )
    .invoke_signed(&[seeds])?;

    Ok(())
}
