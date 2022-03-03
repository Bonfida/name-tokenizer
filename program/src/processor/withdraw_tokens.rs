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
        system_instruction,
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
        };

        // Check keys
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;

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

// Token owner is the main source of truth
// If the NFT is owned by a program then the source of truth
// is the NftRecord owner field

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let nft_record = NftRecord::from_account_info(accounts.nft_record, Tag::ActiveRecord)
        .or_else(|_| NftRecord::from_account_info(accounts.nft_record, Tag::InactiveRecord))?;

    let nft = Account::unpack(&accounts.nft.data.borrow())?;

    if nft.amount != 1 || nft.mint != nft_record.nft_mint {
        msg!("Invalid NFT account");
        return Err(ProgramError::InvalidAccountData);
    }

    let user_owned = *accounts.nft_owner.owner == Pubkey::default();

    if user_owned {
        check_account_key(accounts.nft_owner, &nft.owner)?;
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
    invoke_signed(&ix, &[], &[seeds])?;

    // Withdraw native SOL if any
    let minimum_rent = Rent::get()?.minimum_balance(nft_record.borsh_len());
    let lamports_to_withdraw = accounts
        .nft_record
        .lamports()
        .checked_sub(minimum_rent)
        .unwrap();

    msg!("+ Withdrawing native SOL {}", lamports_to_withdraw);
    let ix = system_instruction::transfer(
        accounts.nft_record.key,
        accounts.nft_owner.key,
        lamports_to_withdraw,
    );
    invoke_signed(&ix, &[], &[seeds])?;

    Ok(())
}
