//! Update the metadata URI and symbol of a tokenized domain name

use crate::mpl_token_metadata::{
    accounts::Metadata,
    instructions::{
        UpdateMetadataAccountV2Cpi, UpdateMetadataAccountV2CpiAccounts,
        UpdateMetadataAccountV2InstructionArgs,
    },
    types::DataV2,
};

use crate::state::{METADATA_SIGNER, META_SYMBOL};

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
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// The new URI of the metadata
    pub uri: String,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The metadata account
    #[cons(writable)]
    pub metadata_account: &'a T,

    /// The central state account
    pub central_state: &'a T,

    /// The metadata program account
    pub metadata_program: &'a T,

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
            central_state: next_account_info(accounts_iter)?,
            metadata_program: next_account_info(accounts_iter)?,
            #[cfg(not(feature = "devnet"))]
            metadata_signer: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;
        check_account_key(accounts.metadata_program, &crate::mpl_token_metadata::ID)?;
        #[cfg(not(feature = "devnet"))]
        check_account_key(accounts.metadata_signer, &METADATA_SIGNER)?;

        // Check owners
        check_account_owner(accounts.metadata_account, &crate::mpl_token_metadata::ID)?;

        // Check signer
        #[cfg(not(feature = "devnet"))]
        check_signer(accounts.metadata_signer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;
    let Params { uri } = params;

    let metadata = Metadata::safe_deserialize(&accounts.metadata_account.data.borrow())?;

    // On-chain metadata strings are puffed out with null bytes
    let name = metadata.name.trim_end_matches('\0').to_string();

    let data = DataV2 {
        name,
        symbol: META_SYMBOL.to_string(),
        uri,
        seller_fee_basis_points: metadata.seller_fee_basis_points,
        creators: metadata.creators,
        collection: metadata.collection,
        uses: metadata.uses,
    };

    let seeds: &[&[u8]] = &[&program_id.to_bytes(), &[crate::central_state::NONCE]];

    UpdateMetadataAccountV2Cpi::new(
        accounts.metadata_program,
        UpdateMetadataAccountV2CpiAccounts {
            metadata: accounts.metadata_account,
            update_authority: accounts.central_state,
        },
        UpdateMetadataAccountV2InstructionArgs {
            data: Some(data),
            new_update_authority: None,
            primary_sale_happened: None,
            is_mutable: None,
        },
    )
    .invoke_signed(&[seeds])?;

    Ok(())
}
