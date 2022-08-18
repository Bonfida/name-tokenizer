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
        pubkey::Pubkey,
        rent::Rent,
        system_program,
        sysvar::Sysvar,
    },
    spl_token::state::Account,
};

use solana_program::program_pack::Pack;

use crate::state::{NftRecord, Tag};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The token account holding the NFT
    #[cons(writable)]
    pub nft: &'a T,

    /// The owner of the NFT token account
    #[cons(writable, signer)]
    pub nft_owner: &'a T,

    /// The NFT record account
    #[cons(writable)]
    pub nft_record: &'a T,

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
            nft: next_account_info(accounts_iter)?,
            nft_owner: next_account_info(accounts_iter)?,
            nft_record: next_account_info(accounts_iter)?,
            token_destination: next_account_info(accounts_iter)?,
            token_source: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;

        // Check owners
        check_account_owner(accounts.nft, &spl_token::ID)?;
        check_account_owner(accounts.nft_record, program_id)?;
        check_account_owner(accounts.token_destination, &spl_token::ID)?;
        check_account_owner(accounts.token_source, &spl_token::ID)?;

        // Check signer
        check_signer(accounts.nft_owner)?;

        Ok(accounts)
    }
}

// NFT record is active -> Correct owner is the token holder
// NFT record is inactive -> Correct owner is the latest person who redeemed

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut nft_record = NftRecord::from_account_info(accounts.nft_record, Tag::ActiveRecord)
        .or_else(|_| NftRecord::from_account_info(accounts.nft_record, Tag::InactiveRecord))?;

    let nft = Account::unpack(&accounts.nft.data.borrow())?;

    if nft.mint != nft_record.nft_mint {
        msg!("+ NFT mint mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if nft_record.is_active() {
        check_account_key(accounts.nft_owner, &nft.owner)?;
        if nft.amount != 1 {
            msg!("+ Invalid NFT amount, received {}", nft.amount);
            return Err(ProgramError::InvalidArgument);
        }
    } else {
        check_account_key(accounts.nft_owner, &nft_record.owner)?
    }

    // Withdraw SPL token
    let token_account = Account::unpack(&accounts.token_source.data.borrow())?;

    msg!("+ Withdrawing tokens {}", token_account.amount);

    let ix = spl_token::instruction::transfer(
        &spl_token::ID,
        accounts.token_source.key,
        accounts.token_destination.key,
        accounts.nft_record.key,
        &[],
        token_account.amount,
    )?;
    let seeds: &[&[u8]] = &[
        NftRecord::SEED,
        &nft_record.name_account.to_bytes(),
        &[nft_record.nonce],
    ];
    invoke_signed(
        &ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.token_source.clone(),
            accounts.token_destination.clone(),
            accounts.nft_record.clone(),
        ],
        &[seeds],
    )?;

    // Withdraw native SOL if any
    let minimum_rent = Rent::get()?.minimum_balance(accounts.nft_record.data_len());
    let lamports_to_withdraw = accounts
        .nft_record
        .lamports()
        .checked_sub(minimum_rent)
        .unwrap();

    msg!("+ Withdrawing native SOL {}", lamports_to_withdraw);
    let mut nft_record_lamports = accounts.nft_record.lamports.borrow_mut();
    let mut nft_owner_lamports = accounts.nft_owner.lamports.borrow_mut();

    **nft_record_lamports -= lamports_to_withdraw;
    **nft_owner_lamports += lamports_to_withdraw;

    // Update NFT record owner
    nft_record.owner = *accounts.nft_owner.key;
    nft_record.save(&mut accounts.nft_record.data.borrow_mut());

    Ok(())
}
