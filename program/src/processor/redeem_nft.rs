//! Redeem a tokenized domain name

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_name_service::instruction::transfer,
    spl_token::instruction::burn,
};

use crate::state::{NftRecord, Tag, MINT_PREFIX};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The mint of the NFT
    #[cons(writable)]
    pub mint: &'a T,

    /// The current token account holding the NFT
    #[cons(writable)]
    pub nft_source: &'a T,

    /// The NFT owner account
    #[cons(writable, signer)]
    pub nft_owner: &'a T,

    /// The NFT record account
    #[cons(writable)]
    pub nft_record: &'a T,

    /// The domain name account
    #[cons(writable)]
    pub name_account: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The SPL name service program account
    pub spl_name_service_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            mint: next_account_info(accounts_iter)?,
            nft_source: next_account_info(accounts_iter)?,
            nft_owner: next_account_info(accounts_iter)?,
            nft_record: next_account_info(accounts_iter)?,
            name_account: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            spl_name_service_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;
        check_account_key(accounts.spl_name_service_program, &spl_name_service::ID)?;

        // Check owners
        check_account_owner(accounts.mint, &spl_token::ID)?;
        check_account_owner(accounts.nft_source, &spl_token::ID)?;
        check_account_owner(accounts.nft_record, program_id)?;
        check_account_owner(accounts.name_account, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.nft_owner)?;

        Ok(accounts)
    }
}
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut nft_record = NftRecord::from_account_info(accounts.nft_record, Tag::ActiveRecord)?;

    let (nft_record_key, _) = NftRecord::find_key(accounts.name_account.key, program_id);
    check_account_key(accounts.nft_record, &nft_record_key)?;

    let (mint, _) = Pubkey::find_program_address(
        &[MINT_PREFIX, &accounts.name_account.key.to_bytes()],
        program_id,
    );
    check_account_key(accounts.mint, &mint)?;

    // Burn NFT
    let ix = burn(
        &spl_token::ID,
        accounts.nft_source.key,
        &nft_record.nft_mint,
        accounts.nft_owner.key,
        &[],
        1,
    )?;
    invoke(
        &ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.nft_source.clone(),
            accounts.mint.clone(),
            accounts.nft_owner.clone(),
        ],
    )?;

    // Transfer domain
    let ix = transfer(
        spl_name_service::ID,
        *accounts.nft_owner.key,
        *accounts.name_account.key,
        *accounts.nft_record.key,
        None,
    )?;
    let seeds: &[&[u8]] = &[
        NftRecord::SEED,
        &accounts.name_account.key.to_bytes(),
        &[nft_record.nonce],
    ];
    invoke_signed(
        &ix,
        &[
            accounts.spl_name_service_program.clone(),
            accounts.nft_owner.clone(),
            accounts.name_account.clone(),
            accounts.nft_record.clone(),
        ],
        &[seeds],
    )?;

    // Update NFT record
    nft_record.tag = Tag::InactiveRecord;
    nft_record.owner = *accounts.nft_owner.key;

    nft_record.save(&mut accounts.nft_record.data.borrow_mut());

    Ok(())
}
