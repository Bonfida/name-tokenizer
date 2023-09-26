//! Unverify an NFT

use crate::state::{COLLECTION_PREFIX, METADATA_SIGNER};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_token_metadata::{
        instruction::unverify_collection,
        pda::{find_master_edition_account, find_metadata_account},
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program::invoke_signed,
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

    let (edition_key, _) = find_master_edition_account(&collection_mint);
    check_account_key(accounts.edition_account, &edition_key)?;

    // Verify collection metadata PDA
    let (collection_metadata, _) = find_metadata_account(&collection_mint);
    check_account_key(accounts.collection_metadata, &collection_metadata)?;

    let seeds: &[&[u8]] = &[&program_id.to_bytes(), &[crate::central_state::NONCE]];

    let ix = unverify_collection(
        mpl_token_metadata::ID,
        *accounts.metadata_account.key,
        crate::central_state::KEY,
        collection_mint,
        collection_metadata,
        edition_key,
        None,
    );
    invoke_signed(
        &ix,
        &[
            accounts.metadata_program.clone(),
            accounts.metadata_account.clone(),
            accounts.central_state.clone(),
            accounts.collection_mint.clone(),
            accounts.collection_metadata.clone(),
            accounts.edition_account.clone(),
        ],
        &[seeds],
    )?;

    Ok(())
}
