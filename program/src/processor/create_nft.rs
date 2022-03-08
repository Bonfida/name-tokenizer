//! Tokenize a domain name

use crate::{
    cpi::Cpi,
    state::{
        NftRecord, Tag, COLLECTION_PREFIX, CREATOR_FEE, META_SYMBOL, MINT_PREFIX, SELLER_BASIS,
    },
    utils::check_name,
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_token_metadata::{
        instruction::{create_metadata_accounts_v2, set_and_verify_collection},
        pda::{find_master_edition_account, find_metadata_account},
        state::Creator,
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_program, sysvar,
    },
    spl_name_service::instruction::transfer,
    spl_token::{instruction::mint_to, state::Mint},
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// The domain name (without .sol)
    pub name: String,

    /// The URI of the metadata
    pub uri: String,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The mint of the NFT
    #[cons(writable)]
    pub mint: &'a T,

    /// The NFT token destination
    #[cons(writable)]
    pub nft_destination: &'a T,

    /// The domain name account
    #[cons(writable)]
    pub name_account: &'a T,

    /// The NFT record account
    #[cons(writable)]
    pub nft_record: &'a T,

    /// The domain name owner
    #[cons(writable, signer)]
    pub name_owner: &'a T,

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
    #[cons(writable)]
    pub fee_payer: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The metadata program account
    pub metadata_program: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The SPL name service program account
    pub spl_name_service_program: &'a T,

    /// Rent sysvar account
    pub rent_account: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            mint: next_account_info(accounts_iter)?,
            nft_destination: next_account_info(accounts_iter)?,
            name_account: next_account_info(accounts_iter)?,
            nft_record: next_account_info(accounts_iter)?,
            name_owner: next_account_info(accounts_iter)?,
            metadata_account: next_account_info(accounts_iter)?,
            edition_account: next_account_info(accounts_iter)?,
            collection_metadata: next_account_info(accounts_iter)?,
            collection_mint: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            metadata_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            spl_name_service_program: next_account_info(accounts_iter)?,
            rent_account: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;
        check_account_key(accounts.metadata_program, &mpl_token_metadata::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_service_program, &spl_name_service::ID)?;
        check_account_key(accounts.rent_account, &sysvar::rent::ID)?;

        // Check owners
        check_account_owner(accounts.mint, &spl_token::ID)?;
        check_account_owner(accounts.nft_destination, &spl_token::ID)?;
        check_account_owner(accounts.name_account, &spl_name_service::ID)?;
        check_account_owner(accounts.nft_record, &system_program::ID)
            .or_else(|_| check_account_owner(accounts.nft_record, program_id))?;
        check_account_owner(accounts.metadata_account, &system_program::ID)
            .or_else(|_| check_account_owner(accounts.metadata_account, &mpl_token_metadata::ID))?;
        check_account_owner(accounts.edition_account, &mpl_token_metadata::ID)?;
        check_account_owner(accounts.collection_metadata, &mpl_token_metadata::ID)?;
        check_account_owner(accounts.collection_mint, &spl_token::ID)?;

        // Check signer
        check_signer(accounts.name_owner)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let Params { name, uri } = params;

    let (mint, _) = Pubkey::find_program_address(
        &[MINT_PREFIX, &accounts.name_account.key.to_bytes()],
        program_id,
    );
    check_account_key(accounts.mint, &mint)?;

    // Create NFT record
    let (nft_record_key, nft_record_nonce) =
        NftRecord::find_key(accounts.name_account.key, program_id);
    check_account_key(accounts.nft_record, &nft_record_key)?;

    // Verify name derivation
    check_name(&name, accounts.name_account)?;

    // Verify metadata PDA
    let (metadata_key, _) = find_metadata_account(&mint);
    check_account_key(accounts.metadata_account, &metadata_key)?;

    // Verifiy edition PDA
    let (collection_mint, _) =
        Pubkey::find_program_address(&[COLLECTION_PREFIX, &program_id.to_bytes()], program_id);
    check_account_key(accounts.collection_mint, &collection_mint)?;

    let (edition_key, _) = find_master_edition_account(&collection_mint);
    check_account_key(accounts.edition_account, &edition_key)?;

    // Verify collection metadata PDA
    let (collection_metadata, _) = find_metadata_account(&collection_mint);
    check_account_key(accounts.collection_metadata, &collection_metadata)?;

    // Verify mint
    let mint_info = Mint::unpack(&accounts.mint.data.borrow())?;
    if mint_info.supply != 0 {
        msg!("Expected suply == 0 and received {}", mint_info.supply);
        return Err(ProgramError::InvalidAccountData);
    }

    if accounts.nft_record.data_is_empty() {
        msg!("+ Creating NFT record");
        let nft_record = NftRecord::new(
            nft_record_nonce,
            *accounts.name_owner.key,
            *accounts.name_account.key,
            mint,
        );
        let seeds: &[&[u8]] = &[
            NftRecord::SEED,
            &accounts.name_account.key.to_bytes(),
            &[nft_record_nonce],
        ];
        Cpi::create_account(
            program_id,
            accounts.system_program,
            accounts.fee_payer,
            accounts.nft_record,
            seeds,
            nft_record.borsh_len(),
        )?;

        nft_record.save(&mut accounts.nft_record.data.borrow_mut());
    } else {
        msg!("+ NFT record already exists");
        let mut nft_record =
            NftRecord::from_account_info(accounts.nft_record, Tag::InactiveRecord)?;

        check_account_key(accounts.mint, &nft_record.nft_mint)?;

        nft_record.tag = Tag::ActiveRecord;
        nft_record.owner = *accounts.name_owner.key;

        nft_record.save(&mut accounts.nft_record.data.borrow_mut());
    }

    // Mint token
    let ix = mint_to(
        &spl_token::ID,
        &mint,
        accounts.nft_destination.key,
        &crate::central_state::KEY,
        &[],
        1,
    )?;
    let seeds: &[&[u8]] = &[&program_id.to_bytes(), &[crate::central_state::NONCE]];

    invoke_signed(
        &ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.mint.clone(),
            accounts.nft_destination.clone(),
            accounts.central_state.clone(),
        ],
        &[seeds],
    )?;

    // Create metadata
    if accounts.metadata_account.data_is_empty() {
        msg!("+ Creating metadata");
        let central_creator = Creator {
            address: crate::central_state::KEY,
            verified: true,
            share: 0,
        };
        let (collection_mint, _) =
            Pubkey::find_program_address(&[COLLECTION_PREFIX, &program_id.to_bytes()], program_id);
        let (collection, _) = find_metadata_account(&collection_mint);

        let ix = create_metadata_accounts_v2(
            mpl_token_metadata::ID,
            *accounts.metadata_account.key,
            mint,
            crate::central_state::KEY,
            *accounts.fee_payer.key,
            crate::central_state::KEY,
            name,
            META_SYMBOL.to_string(),
            uri,
            Some(vec![central_creator, CREATOR_FEE]),
            SELLER_BASIS,
            true,
            true,
            None,
            None,
        );
        invoke_signed(
            &ix,
            &[
                accounts.metadata_program.clone(),
                accounts.metadata_account.clone(),
                accounts.rent_account.clone(),
                accounts.mint.clone(),
                accounts.central_state.clone(),
                accounts.fee_payer.clone(),
            ],
            &[seeds],
        )?;

        msg!("+ Verifying collection");
        let ix = set_and_verify_collection(
            mpl_token_metadata::ID,
            metadata_key,
            crate::central_state::KEY,
            *accounts.fee_payer.key,
            crate::central_state::KEY,
            collection_mint,
            collection,
            edition_key,
            None,
        );
        invoke_signed(
            &ix,
            &[
                accounts.metadata_program.clone(),
                accounts.metadata_account.clone(),
                accounts.central_state.clone(),
                accounts.fee_payer.clone(),
                accounts.central_state.clone(),
                accounts.collection_mint.clone(),
                accounts.collection_metadata.clone(),
                accounts.edition_account.clone(),
            ],
            &[seeds],
        )?;
    } else {
        msg!("+ Metadata already exists");
        // TODO maybe update metadata to override what could have been written in the account?
    }

    // Transfer domain
    let ix = transfer(
        spl_name_service::ID,
        nft_record_key,
        *accounts.name_account.key,
        *accounts.name_owner.key,
        None,
    )?;
    invoke(
        &ix,
        &[
            accounts.spl_name_service_program.clone(),
            accounts.nft_record.clone(),
            accounts.name_account.clone(),
            accounts.name_owner.clone(),
        ],
    )?;

    Ok(())
}
