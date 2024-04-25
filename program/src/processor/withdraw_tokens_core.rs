//! Withdraw funds that have been sent to the escrow
//! while the domain was tokenized

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
        rent::Rent,
        system_program,
        sysvar::Sysvar,
    },
    spl_token::state::Account,
};

use crate::{
    error::TokenizerError,
    state::{CoreRecord, Tag},
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The token account holding the NFT
    #[cons(writable)]
    pub core_asset: &'a T,

    /// The owner of the NFT token account
    #[cons(writable, signer)]
    pub core_asset_owner: &'a T,

    /// The MPL Core record account
    #[cons(writable)]
    pub core_record: &'a T,

    /// The destination for tokens being withdrawn
    #[cons(writable)]
    pub token_destination: &'a T,

    /// The source for tokens being withdrawn
    #[cons(writable)]
    pub token_source: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The system program account
    pub system_program: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            core_asset: next_account_info(accounts_iter)?,
            core_asset_owner: next_account_info(accounts_iter)?,
            core_record: next_account_info(accounts_iter)?,
            token_destination: next_account_info(accounts_iter)?,
            token_source: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;

        // Check owners
        check_account_owner(accounts.core_asset, &mpl_core::ID)?;
        check_account_owner(accounts.core_record, program_id)?;
        check_account_owner(accounts.token_destination, &spl_token::ID)?;
        check_account_owner(accounts.token_source, &spl_token::ID)?;

        // Check signer
        check_signer(accounts.core_asset_owner)?;

        Ok(accounts)
    }
}

// NFT record is active -> Correct owner is the token holder
// NFT record is inactive -> Correct owner is the latest person who redeemed

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut core_record =
        CoreRecord::from_account_info(accounts.core_record, Tag::ActiveCoreRecord).or_else(
            |_| CoreRecord::from_account_info(accounts.core_record, Tag::InactiveCoreRecord),
        )?;

    let asset = mpl_core::Asset::from_bytes(&accounts.core_asset.data.borrow())?;

    if core_record.core_asset != *accounts.core_asset.key {
        return Err(TokenizerError::CoreAssetMistmatch.into());
    }

    if core_record.is_active() {
        if asset.base.owner != *accounts.core_asset_owner.key {
            return Err(TokenizerError::CoreAssetOwnerMismatch.into());
        }
    } else {
        check_account_key(accounts.core_asset_owner, &core_record.owner)?;
    }

    // Withdraw SPL token
    let token_account = Account::unpack(&accounts.token_source.data.borrow())?;

    msg!("[+] Withdrawing tokens {}", token_account.amount);

    let ix = spl_token::instruction::transfer(
        &spl_token::ID,
        accounts.token_source.key,
        accounts.token_destination.key,
        accounts.core_record.key,
        &[],
        token_account.amount,
    )?;
    let seeds: &[&[u8]] = &[
        CoreRecord::SEED,
        &core_record.name_account.to_bytes(),
        &[core_record.nonce],
    ];
    invoke_signed(
        &ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.token_source.clone(),
            accounts.token_destination.clone(),
            accounts.core_record.clone(),
        ],
        &[seeds],
    )?;

    // Withdraw native SOL if any
    let minimum_rent = Rent::get()?.minimum_balance(accounts.core_record.data_len());
    let lamports_to_withdraw = accounts
        .core_record
        .lamports()
        .checked_sub(minimum_rent)
        .unwrap();

    msg!("[+] Withdrawing native SOL {}", lamports_to_withdraw);
    let mut nft_record_lamports = accounts.core_record.lamports.borrow_mut();
    let mut nft_owner_lamports = accounts.core_asset_owner.lamports.borrow_mut();

    **nft_record_lamports -= lamports_to_withdraw;
    **nft_owner_lamports += lamports_to_withdraw;

    // Update NFT record owner
    core_record.owner = *accounts.core_asset_owner.key;
    core_record.save(&mut accounts.core_record.data.borrow_mut());

    Ok(())
}
