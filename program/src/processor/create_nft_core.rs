//! Tokenize a domain name

use crate::{
    cpi::Cpi,
    error::TokenizerError,
    state::{CoreRecord, Tag, CORE_ASSET_PREFIX, CREATOR_KEY, METADATA_CORE_SIGNER, SELLER_BASIS},
    utils::{self, check_name, get_plugins},
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_core::types::{PermanentFreezeDelegate, Royalties},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
    spl_name_service::instruction::transfer,
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
    pub core_asset: &'a T,

    /// The domain name account
    #[cons(writable)]
    pub name_account: &'a T,

    #[cons(writable)]
    pub collection: &'a T,

    /// The NFT record account
    #[cons(writable)]
    pub core_record: &'a T,

    /// The domain name owner
    #[cons(writable, signer)]
    pub name_owner: &'a T,

    /// The central state account
    pub central_state: &'a T,

    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The metadata program account
    pub mpl_core_program: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The SPL name service program account
    pub spl_name_service_program: &'a T,

    /// The metadata signer
    #[cons(signer)]
    #[cfg(not(feature = "devnet"))]
    pub metadata_signer: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            core_asset: next_account_info(accounts_iter)?,
            name_account: next_account_info(accounts_iter)?,
            collection: next_account_info(accounts_iter)?,
            core_record: next_account_info(accounts_iter)?,
            name_owner: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            mpl_core_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            spl_name_service_program: next_account_info(accounts_iter)?,
            #[cfg(not(feature = "devnet"))]
            metadata_signer: next_account_info(accounts_iter).unwrap(),
        };

        // Check keys
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;
        check_account_key(accounts.mpl_core_program, &mpl_core::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_service_program, &spl_name_service::ID)?;
        #[cfg(not(feature = "devnet"))]
        check_account_key(accounts.metadata_signer, &METADATA_CORE_SIGNER)?;

        // Check owners
        check_account_owner(accounts.core_asset, &mpl_core::ID)
            .or_else(|_| check_account_owner(accounts.core_asset, &system_program::ID))?;
        check_account_owner(accounts.name_account, &spl_name_service::ID)?;
        check_account_owner(accounts.collection, &mpl_core::ID)?;
        check_account_owner(accounts.core_record, &system_program::ID)
            .or_else(|_| check_account_owner(accounts.core_record, program_id))?;

        // Check signer
        check_signer(accounts.name_owner)?;
        #[cfg(not(feature = "devnet"))]
        check_signer(accounts.metadata_signer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    let Params { name, uri } = params;

    let (asset_key, asset_nonce) = utils::get_core_nft_key(accounts.name_account.key);
    check_account_key(accounts.core_asset, &asset_key)?;

    let (collection_key, _) = utils::get_core_collection_key();
    check_account_key(accounts.collection, &collection_key)?;

    // Create NFT record
    let (core_record_key, core_record_nonce) =
        CoreRecord::find_key(accounts.name_account.key, program_id);
    check_account_key(accounts.core_record, &core_record_key)?;

    // Verify name derivation
    check_name(&name, accounts.name_account)?;

    if accounts.core_record.data_is_empty() {
        msg!("[+] Creating Core record");

        let core_record = CoreRecord::new(
            core_record_nonce,
            *accounts.name_owner.key,
            *accounts.name_account.key,
            asset_key,
        );
        let seeds: &[&[u8]] = &[
            CoreRecord::SEED,
            accounts.name_account.key.as_ref(),
            &[core_record_nonce],
        ];
        Cpi::create_account(
            program_id,
            accounts.system_program,
            accounts.fee_payer,
            accounts.core_record,
            seeds,
            core_record.borsh_len(),
        )?;

        core_record.save(&mut accounts.core_record.data.borrow_mut());
    } else {
        msg!("[+] Core record already exists");
        let mut core_record =
            CoreRecord::from_account_info(accounts.core_record, Tag::InactiveCoreRecord)?;
        check_account_key(accounts.core_asset, &core_record.core_asset)?;

        core_record.tag = Tag::ActiveCoreRecord;
        core_record.owner = *accounts.name_owner.key;

        core_record.save(&mut accounts.core_record.data.borrow_mut());
    }

    // Create MPL Core asset
    if accounts.core_asset.data_is_empty() {
        msg!("[+] Creating MPL Core asset");
        mpl_core::instructions::CreateV1Cpi::new(
            accounts.mpl_core_program,
            mpl_core::instructions::CreateV1CpiAccounts {
                asset: accounts.core_asset,
                collection: Some(accounts.collection),
                authority: Some(accounts.central_state),
                payer: accounts.fee_payer,
                owner: Some(accounts.name_owner),
                update_authority: None,
                system_program: accounts.system_program,
                log_wrapper: None,
            },
            mpl_core::instructions::CreateV1InstructionArgs {
                uri,
                name,
                data_state: mpl_core::types::DataState::AccountState,
                plugins: Some(get_plugins()),
            },
        )
        .invoke_signed(&[
            &[
                CORE_ASSET_PREFIX,
                accounts.name_account.key.as_ref(),
                &[asset_nonce],
            ],
            &[crate::ID.as_ref(), &[crate::central_state::NONCE]],
        ])?;
    } else {
        msg!("[+] MPL Core asset exists");

        let core_asset = mpl_core::Asset::from_bytes(&accounts.core_asset.data.borrow())?;

        // Must be frozen and owned by escrow
        let is_frozen = core_asset
            .plugin_list
            .permanent_freeze_delegate
            .unwrap()
            .permanent_freeze_delegate
            .frozen;

        let owner = core_asset.base.owner;

        if !is_frozen || owner != crate::central_state::KEY {
            return Err(TokenizerError::InvalidCoreAssetState.into());
        }

        let seeds: &[&[u8]] = &[&program_id.as_ref(), &[crate::central_state::NONCE]];

        // Update asset (i.e metadata)
        mpl_core::instructions::UpdateV1Cpi::new(
            accounts.mpl_core_program,
            mpl_core::instructions::UpdateV1CpiAccounts {
                asset: accounts.core_asset,
                payer: accounts.fee_payer,
                authority: Some(accounts.central_state),
                collection: Some(accounts.collection),
                log_wrapper: None,
                system_program: accounts.system_program,
            },
            mpl_core::instructions::UpdateV1InstructionArgs {
                new_name: Some(name),
                new_update_authority: None,
                new_uri: Some(uri),
            },
        )
        .invoke_signed(&[seeds])?;

        // Unfreeze
        mpl_core::instructions::UpdatePluginV1Cpi::new(
            accounts.mpl_core_program,
            mpl_core::instructions::UpdatePluginV1CpiAccounts {
                asset: accounts.core_asset,
                collection: Some(accounts.collection),
                authority: Some(accounts.central_state),
                payer: accounts.fee_payer,
                system_program: accounts.system_program,
                log_wrapper: None,
            },
            mpl_core::instructions::UpdatePluginV1InstructionArgs {
                plugin: mpl_core::types::Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate {
                    frozen: false,
                }),
            },
        )
        .invoke_signed(&[seeds])?;

        // Updat royalties to allow dynamic royalties
        mpl_core::instructions::UpdatePluginV1Cpi::new(
            accounts.mpl_core_program,
            mpl_core::instructions::UpdatePluginV1CpiAccounts {
                asset: accounts.core_asset,
                collection: Some(accounts.collection),
                authority: Some(accounts.central_state),
                payer: accounts.fee_payer,
                system_program: accounts.system_program,
                log_wrapper: None,
            },
            mpl_core::instructions::UpdatePluginV1InstructionArgs {
                plugin: mpl_core::types::Plugin::Royalties(Royalties {
                    basis_points: SELLER_BASIS,
                    creators: vec![mpl_core::types::Creator {
                        address: CREATOR_KEY,
                        percentage: 100,
                    }],
                    rule_set: mpl_core::types::RuleSet::None,
                }),
            },
        )
        .invoke_signed(&[seeds])?;

        // Transfer
        mpl_core::instructions::TransferV1Cpi::new(
            accounts.mpl_core_program,
            mpl_core::instructions::TransferV1CpiAccounts {
                asset: accounts.core_asset,
                collection: Some(accounts.collection),
                payer: accounts.fee_payer,
                authority: Some(accounts.central_state),
                new_owner: accounts.name_owner,
                system_program: None,
                log_wrapper: None,
            },
            mpl_core::instructions::TransferV1InstructionArgs {
                compression_proof: None,
            },
        )
        .invoke_signed(&[seeds])?;
    }

    // Transfer domain
    let ix = transfer(
        spl_name_service::ID,
        core_record_key,
        *accounts.name_account.key,
        *accounts.name_owner.key,
        None,
    )?;
    invoke(
        &ix,
        &[
            accounts.spl_name_service_program.clone(),
            accounts.core_record.clone(),
            accounts.name_account.clone(),
            accounts.name_owner.clone(),
        ],
    )?;

    Ok(())
}
