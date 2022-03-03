//! Tokenize a domain name

use solana_program::program_pack::Pack;

use crate::{
    cpi::Cpi,
    state::{CentralState, NftRecord, Tag, META_SYMBOL, MINT_PREFIX, SELLER_BASIS},
    utils::check_name,
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_token_metadata::{
        instruction::create_metadata_accounts_v2, pda::find_metadata_account, state::Creator,
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program, sysvar,
    },
    spl_associated_token_account::create_associated_token_account,
    spl_name_service::instruction::transfer,
    spl_token::{
        instruction::{initialize_mint, mint_to},
        state::Mint,
    },
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

    /// The central state account
    pub central_state: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The metadata program account
    pub metadata_program: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The SPL name service program account
    pub spl_name_service_program: &'a T,

    /// Associated token account program
    pub ata_program: &'a T,

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
            central_state: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            metadata_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            spl_name_service_program: next_account_info(accounts_iter)?,
            ata_program: next_account_info(accounts_iter)?,
            rent_account: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.spl_token_program, &spl_token::ID).unwrap();
        check_account_key(accounts.system_program, &system_program::ID).unwrap();
        check_account_key(accounts.spl_name_service_program, &spl_name_service::ID).unwrap();
        check_account_key(accounts.ata_program, &spl_associated_token_account::ID).unwrap();
        check_account_key(accounts.rent_account, &sysvar::rent::ID).unwrap();

        // Check owners
        check_account_owner(accounts.mint, &system_program::ID)
            .or_else(|_| check_account_owner(accounts.mint, &spl_token::ID))
            .unwrap();
        check_account_owner(accounts.nft_destination, &system_program::ID)
            .or_else(|_| check_account_owner(accounts.nft_destination, &spl_token::ID))
            .unwrap();
        check_account_owner(accounts.name_account, &spl_name_service::ID).unwrap();
        check_account_owner(accounts.nft_record, &system_program::ID)
            .or_else(|_| check_account_owner(accounts.nft_record, program_id))
            .unwrap();
        check_account_owner(accounts.metadata_account, &system_program::ID)
            .or_else(|_| check_account_owner(accounts.metadata_account, &mpl_token_metadata::ID))
            .unwrap();
        check_account_owner(accounts.central_state, program_id).unwrap();

        // Check signer
        check_signer(accounts.name_owner)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let Params { name, uri } = params;

    let (central_key, central_nonce) = CentralState::find_key(program_id);
    check_account_key(accounts.central_state, &central_key)?;

    let (mint, mint_nonce) = Pubkey::find_program_address(
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
            accounts.name_owner,
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

    // Create mint
    if accounts.mint.data_is_empty() {
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
            accounts.name_owner,
            accounts.mint,
            seeds,
            Mint::LEN,
        )?;

        // Initialize mint
        let ix = initialize_mint(&spl_token::ID, &mint, &central_key, Some(&central_key), 0)?;
        invoke_signed(
            &ix,
            &[
                accounts.spl_token_program.clone(),
                accounts.mint.clone(),
                accounts.rent_account.clone(),
            ],
            &[seeds],
        )?;

        // A token account cannot be initialized before the mint
        msg!("+ Creating user ATA");
        let ix = create_associated_token_account(
            accounts.name_owner.key,
            accounts.name_owner.key,
            accounts.mint.key,
        );
        invoke(
            &ix,
            &[
                accounts.ata_program.clone(),
                accounts.spl_token_program.clone(),
                accounts.rent_account.clone(),
                accounts.name_owner.clone(),
                accounts.nft_destination.clone(),
                accounts.mint.clone(),
            ],
        )?;
    } else {
        msg!("+ Mint already exists");
        let mint_info = Mint::unpack(&accounts.mint.data.borrow())?;

        // If the mint already exists the supply should be 0
        if mint_info.supply != 0 {
            msg!("Expected suply == 0 and received {}", mint_info.supply);
            return Err(ProgramError::InvalidAccountData);
        }
    }

    // Mint token
    let ix = mint_to(
        &spl_token::ID,
        &mint,
        accounts.nft_destination.key,
        &central_key,
        &[],
        1,
    )?;
    let seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_nonce]];

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
        let creator = Creator {
            address: central_key,
            verified: true,
            share: 100,
        };
        let ix = create_metadata_accounts_v2(
            mpl_token_metadata::ID,
            *accounts.metadata_account.key,
            mint,
            central_key,
            *accounts.name_owner.key,
            central_key,
            name,
            META_SYMBOL.to_string(),
            uri,
            Some(vec![creator]),
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
                accounts.name_owner.clone(),
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
