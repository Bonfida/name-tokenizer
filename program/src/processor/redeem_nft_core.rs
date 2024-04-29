//! Redeem a tokenized domain name

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_core::types::PermanentFreezeDelegate,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
    spl_name_service::instruction::transfer,
};

use crate::state::{CoreRecord, Tag};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    // The MPL Core asset being redeemed
    #[cons(writable)]
    pub core_asset: &'a T,

    /// The MPL Core asset owner account
    #[cons(writable, signer)]
    pub core_asset_owner: &'a T,

    #[cons(writable)]
    pub collection: &'a T,

    /// The Core record account
    #[cons(writable)]
    pub core_record: &'a T,

    /// The program central state
    pub central_state: &'a T,

    /// The domain name account
    #[cons(writable)]
    pub name_account: &'a T,

    /// The MPL Core program account
    pub mpl_core_program: &'a T,

    /// The SPL name service program account
    pub spl_name_service_program: &'a T,

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
            collection: next_account_info(accounts_iter)?,
            core_record: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            name_account: next_account_info(accounts_iter)?,
            mpl_core_program: next_account_info(accounts_iter)?,
            spl_name_service_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;
        check_account_key(accounts.mpl_core_program, &mpl_core::ID)?;
        check_account_key(accounts.spl_name_service_program, &spl_name_service::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;

        // Check owners
        check_account_owner(accounts.core_asset, &mpl_core::ID)?;
        check_account_owner(accounts.collection, &mpl_core::ID)?;
        check_account_owner(accounts.core_record, program_id)?;
        check_account_owner(accounts.name_account, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.core_asset_owner)?;

        Ok(accounts)
    }
}
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let mut core_record =
        CoreRecord::from_account_info(accounts.core_record, Tag::ActiveCoreRecord)?;

    check_account_key(accounts.core_asset, &core_record.core_asset)?;

    let (core_record_key, _) = CoreRecord::find_key(accounts.name_account.key, program_id);
    check_account_key(accounts.core_record, &core_record_key)?;

    let (core_asset, _) = crate::utils::get_core_nft_key(accounts.name_account.key);
    check_account_key(accounts.core_asset, &core_asset)?;

    let (collection_key, _) = crate::utils::get_core_collection_key();
    check_account_key(accounts.collection, &collection_key)?;

    // Transfer MPL Core asset
    msg!("[+] Transfer Core asset");
    mpl_core::instructions::TransferV1Cpi::new(
        accounts.mpl_core_program,
        mpl_core::instructions::TransferV1CpiAccounts {
            asset: accounts.core_asset,
            collection: Some(accounts.collection),
            payer: accounts.core_asset_owner,
            authority: None,
            new_owner: accounts.central_state,
            log_wrapper: None,
            system_program: None,
        },
        mpl_core::instructions::TransferV1InstructionArgs {
            compression_proof: None,
        },
    )
    .invoke()?;
    // Freeze MPL Core asset
    msg!("[+] Freeze Core asset");
    mpl_core::instructions::UpdatePluginV1Cpi::new(
        accounts.mpl_core_program,
        mpl_core::instructions::UpdatePluginV1CpiAccounts {
            asset: accounts.core_asset,
            collection: Some(accounts.collection),
            payer: accounts.core_asset_owner,
            authority: Some(accounts.central_state),
            system_program: accounts.system_program,
            log_wrapper: None,
        },
        mpl_core::instructions::UpdatePluginV1InstructionArgs {
            plugin: mpl_core::types::Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate {
                frozen: true,
            }),
        },
    )
    .invoke_signed(&[&[crate::ID.as_ref(), &[crate::central_state::NONCE]]])?;

    // Transfer domain
    msg!("[+] Transfer domain");
    let ix = transfer(
        spl_name_service::ID,
        *accounts.core_asset_owner.key,
        *accounts.name_account.key,
        *accounts.core_record.key,
        None,
    )?;
    let seeds: &[&[u8]] = &[
        CoreRecord::SEED,
        &accounts.name_account.key.to_bytes(),
        &[core_record.nonce],
    ];
    invoke_signed(
        &ix,
        &[
            accounts.spl_name_service_program.clone(),
            accounts.core_asset_owner.clone(),
            accounts.name_account.clone(),
            accounts.core_record.clone(),
        ],
        &[seeds],
    )?;

    // Update Core record
    core_record.tag = Tag::InactiveCoreRecord;
    core_record.owner = *accounts.core_asset_owner.key;

    core_record.save(&mut accounts.core_record.data.borrow_mut());

    Ok(())
}
