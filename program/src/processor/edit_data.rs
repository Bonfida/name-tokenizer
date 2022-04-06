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
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

use spl_name_service::instruction::update;

use crate::state::NftRecord;

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// Offset at which the data should be written into the domain name registry
    pub offset: u32,
    /// The data to be written into the registry (overwrites any previous data)
    pub data: Vec<u8>,
}

#[derive(InstructionsAccount, Debug)]
pub struct Accounts<'a, T> {
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
        check_account_owner(accounts.nft_record, program_id)?;
        check_account_owner(accounts.name_account, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.nft_owner)?;

        Ok(accounts)
    }
}
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let (nft_record_key, nft_record_nonce) =
        NftRecord::find_key(accounts.name_account.key, program_id);
    check_account_key(accounts.nft_record, &nft_record_key)?;

    // Transfer domain
    let ix = update(
        spl_name_service::ID,
        params.offset,
        params.data,
        *accounts.name_account.key,
        *accounts.nft_record.key,
        None,
    )?;
    let seeds: &[&[u8]] = &[
        NftRecord::SEED,
        &accounts.name_account.key.to_bytes(),
        &[nft_record_nonce],
    ];
    invoke_signed(
        &ix,
        &[
            accounts.spl_name_service_program.clone(),
            accounts.nft_owner.clone(),
            accounts.nft_record.clone(),
            accounts.name_account.clone(),
        ],
        &[seeds],
    )?;

    Ok(())
}
