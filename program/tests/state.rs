use {
    borsh::BorshSerialize,
    name_tokenizer::{
        entrypoint::process_instruction,
        state::{METADATA_SIGNER, ROOT_DOMAIN_ACCOUNT},
    },
    solana_program::{hash::hashv, pubkey::Pubkey, system_program},
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{
        account::Account,
        program_pack::Pack,
        signer::{keypair::Keypair, Signer},
    },
    spl_associated_token_account::{
        get_associated_token_address, instruction::create_associated_token_account,
    },
    spl_name_service::state::{get_seeds_and_key, HASH_PREFIX},
};

pub mod common;

use borsh::BorshDeserialize;
use mpl_core::{
    types::{Key, UpdateAuthority},
    Asset, Collection,
};
use name_tokenizer::{
    instruction::{create_collection_core, create_nft_core, redeem_nft_core, withdraw_tokens_core},
    state::{CoreRecord, Tag, COLLECTION_CORE_NAME, COLLECTION_CORE_URI},
};
use solana_sdk::system_instruction;

use crate::common::utils::{mint_bootstrap, sign_send_instructions};

#[tokio::test]
async fn test_mpl_core() {
    // Create program and test environment
    let alice = Keypair::new();
    let mint_authority = Keypair::new();

    let mut program_test = ProgramTest::new(
        "name_tokenizer",
        name_tokenizer::ID,
        processor!(process_instruction),
    );
    program_test.add_program("spl_name_service", spl_name_service::ID, None);
    program_test.add_program("core", mpl_core::ID, None);

    // Create domain name
    let name = "something_domain_name";
    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec();

    let (name_key, _) = get_seeds_and_key(
        &spl_name_service::ID,
        hashed_name,
        None,
        Some(&ROOT_DOMAIN_ACCOUNT),
    );

    let name_domain_data = [
        spl_name_service::state::NameRecordHeader {
            parent_name: ROOT_DOMAIN_ACCOUNT,
            owner: alice.pubkey(),
            class: Pubkey::default(),
        }
        .try_to_vec()
        .unwrap(),
        vec![0; 1000],
    ]
    .concat();

    program_test.add_account(
        name_key,
        Account {
            lamports: 1_000_000,
            data: name_domain_data,
            owner: spl_name_service::id(),
            ..Account::default()
        },
    );

    //
    // Create mint
    //
    let (usdc_mint, _) = mint_bootstrap(None, 6, &mut program_test, &mint_authority.pubkey());

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    let fee_payer = &prg_test_ctx.payer.pubkey();

    ///////////////////////////////////////////////////
    // Create collection
    ///////////////////////////////////////////////////
    let (collection, _) = name_tokenizer::utils::get_core_collection_key();
    let ix = create_collection_core(
        create_collection_core::Accounts {
            central_state: &name_tokenizer::central_state::KEY,
            fee_payer,
            mpl_core_program: &mpl_core::ID,
            system_program: &system_program::ID,
            collection: &collection,
        },
        create_collection_core::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let data = prg_test_ctx
        .banks_client
        .get_account(collection)
        .await
        .unwrap()
        .unwrap()
        .data;
    let fetched_collection = Collection::from_bytes(&data).unwrap();
    let plugin_header = fetched_collection.plugin_header.unwrap();
    let plugin_list = fetched_collection.plugin_list;

    // Verify base
    assert_eq!(fetched_collection.base.key, Key::CollectionV1);
    assert_eq!(fetched_collection.base.name, COLLECTION_CORE_NAME);
    assert_eq!(fetched_collection.base.num_minted, 0);
    assert_eq!(
        fetched_collection.base.update_authority,
        name_tokenizer::central_state::KEY
    );
    assert_eq!(fetched_collection.base.uri, COLLECTION_CORE_URI);

    // Verify plugin header
    assert_eq!(plugin_header.key, Key::PluginHeaderV1);
    assert_eq!(plugin_header.plugin_registry_offset, 127);

    assert!(plugin_list.attributes.is_none());
    assert!(plugin_list.freeze_delegate.is_none());
    assert!(plugin_list.burn_delegate.is_none());
    assert!(plugin_list.transfer_delegate.is_none());
    assert!(plugin_list.update_delegate.is_none());
    assert!(plugin_list.edition.is_none());
    assert!(plugin_list.permanent_burn_delegate.is_none());

    assert_eq!(
        plugin_list
            .permanent_freeze_delegate
            .clone()
            .unwrap()
            .base
            .authority
            .address
            .unwrap(),
        name_tokenizer::central_state::KEY
    );
    assert_eq!(
        plugin_list
            .permanent_freeze_delegate
            .clone()
            .unwrap()
            .permanent_freeze_delegate
            .frozen,
        false
    );
    assert_eq!(
        plugin_list
            .permanent_transfer_delegate
            .clone()
            .unwrap()
            .base
            .authority
            .address
            .unwrap(),
        name_tokenizer::central_state::KEY
    );

    ///////////////////////////////////////////////////
    // Create NFT (Core Asset)
    ///////////////////////////////////////////////////
    let (core_asset, _) = name_tokenizer::utils::get_core_nft_key(&name_key);
    let (core_record, _) = CoreRecord::find_key(&name_key, &name_tokenizer::ID);
    let uri = "https://...";
    let ix = create_nft_core(
        create_nft_core::Accounts {
            core_asset: &core_asset,
            name_account: &name_key,
            collection: &collection,
            core_record: &core_record,
            name_owner: &alice.pubkey(),
            central_state: &name_tokenizer::central_state::KEY,
            fee_payer,
            mpl_core_program: &mpl_core::ID,
            system_program: &system_program::ID,
            spl_name_service_program: &spl_name_service::ID,
            #[cfg(not(feature = "devnet"))]
            metadata_signer: &METADATA_SIGNER,
        },
        create_nft_core::Params {
            name: name.to_owned(),
            uri: uri.to_owned(),
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    let data = prg_test_ctx
        .banks_client
        .get_account(core_asset)
        .await
        .unwrap()
        .unwrap()
        .data;
    let fetched_core_asset = Asset::from_bytes(&data).unwrap();
    let plugin_list = fetched_core_asset.plugin_list;
    let plugin_header = fetched_core_asset.plugin_header.unwrap();

    assert_eq!(fetched_core_asset.base.key, Key::AssetV1);
    assert_eq!(fetched_core_asset.base.name, name);
    assert_eq!(fetched_core_asset.base.uri, uri);
    assert_eq!(fetched_core_asset.base.owner, alice.pubkey());
    assert_eq!(
        fetched_core_asset.base.update_authority,
        UpdateAuthority::Collection(collection)
    );
    assert!(fetched_core_asset.base.seq.is_none());

    let data = prg_test_ctx
        .banks_client
        .get_account(core_record)
        .await
        .unwrap()
        .unwrap()
        .data;
    let fetched_core_record = CoreRecord::deserialize(&mut data.as_slice()).unwrap();
    assert_eq!(fetched_core_record.core_asset, core_asset);
    assert_eq!(fetched_core_record.name_account, name_key);
    assert!(matches!(fetched_core_record.tag, Tag::ActiveCoreRecord));
    assert_eq!(fetched_core_record.owner, alice.pubkey());

    // Verify plugin header
    assert_eq!(plugin_header.key, Key::PluginHeaderV1);
    assert_eq!(plugin_header.plugin_registry_offset, 160);

    assert!(plugin_list.attributes.is_none());
    assert!(plugin_list.freeze_delegate.is_none());
    assert!(plugin_list.burn_delegate.is_none());
    assert!(plugin_list.transfer_delegate.is_none());
    assert!(plugin_list.update_delegate.is_none());
    assert!(plugin_list.edition.is_none());
    assert!(plugin_list.permanent_burn_delegate.is_none());

    assert_eq!(
        plugin_list
            .permanent_freeze_delegate
            .clone()
            .unwrap()
            .base
            .authority
            .address
            .unwrap(),
        name_tokenizer::central_state::KEY
    );
    assert_eq!(
        plugin_list
            .permanent_freeze_delegate
            .clone()
            .unwrap()
            .permanent_freeze_delegate
            .frozen,
        false
    );
    assert_eq!(
        plugin_list
            .permanent_transfer_delegate
            .clone()
            .unwrap()
            .base
            .authority
            .address
            .unwrap(),
        name_tokenizer::central_state::KEY
    );

    ///////////////////////////////////////////////////
    // Redeem NFT (Core Asset)
    ///////////////////////////////////////////////////

    let ix = redeem_nft_core(
        redeem_nft_core::Accounts {
            core_asset: &core_asset,
            core_asset_owner: &alice.pubkey(),
            collection: &collection,
            core_record: &core_record,
            name_account: &name_key,
            central_state: &name_tokenizer::central_state::KEY,
            mpl_core_program: &mpl_core::ID,
            spl_name_service_program: &spl_name_service::ID,
            system_program: &system_program::ID,
        },
        redeem_nft_core::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    let data = prg_test_ctx
        .banks_client
        .get_account(core_record)
        .await
        .unwrap()
        .unwrap()
        .data;
    let fetched_core_record = CoreRecord::deserialize(&mut data.as_slice()).unwrap();
    assert_eq!(fetched_core_record.core_asset, core_asset);
    assert_eq!(fetched_core_record.name_account, name_key);
    assert!(matches!(fetched_core_record.tag, Tag::InactiveCoreRecord));
    assert_eq!(fetched_core_record.owner, alice.pubkey());

    let data = prg_test_ctx
        .banks_client
        .get_account(core_asset)
        .await
        .unwrap()
        .unwrap()
        .data;
    let fetched_core_asset = Asset::from_bytes(&data).unwrap();
    let plugin_list = fetched_core_asset.plugin_list;

    assert_eq!(
        fetched_core_asset.base.owner,
        name_tokenizer::central_state::KEY
    );
    assert_eq!(
        plugin_list
            .permanent_freeze_delegate
            .clone()
            .unwrap()
            .base
            .authority
            .address
            .unwrap(),
        name_tokenizer::central_state::KEY
    );
    assert_eq!(
        plugin_list
            .permanent_freeze_delegate
            .clone()
            .unwrap()
            .permanent_freeze_delegate
            .frozen,
        true
    );
    assert_eq!(
        plugin_list
            .permanent_transfer_delegate
            .clone()
            .unwrap()
            .base
            .authority
            .address
            .unwrap(),
        name_tokenizer::central_state::KEY
    );

    ///////////////////////////////////////////////////
    // Recreate NFT (Core Asset)
    ///////////////////////////////////////////////////
    let uri = "some_new_uri";
    let (core_asset, _) = name_tokenizer::utils::get_core_nft_key(&name_key);
    let (core_record, _) = CoreRecord::find_key(&name_key, &name_tokenizer::ID);
    let ix = create_nft_core(
        create_nft_core::Accounts {
            core_asset: &core_asset,
            name_account: &name_key,
            collection: &collection,
            core_record: &core_record,
            name_owner: &alice.pubkey(),
            central_state: &name_tokenizer::central_state::KEY,
            fee_payer,
            mpl_core_program: &mpl_core::ID,
            system_program: &system_program::ID,
            spl_name_service_program: &spl_name_service::ID,
            #[cfg(not(feature = "devnet"))]
            metadata_signer: &METADATA_SIGNER,
        },
        create_nft_core::Params {
            name: name.to_owned(),
            uri: uri.to_owned(),
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    let data = prg_test_ctx
        .banks_client
        .get_account(core_asset)
        .await
        .unwrap()
        .unwrap()
        .data;
    let fetched_core_asset = Asset::from_bytes(&data).unwrap();
    let plugin_list = fetched_core_asset.plugin_list;

    assert_eq!(fetched_core_asset.base.key, Key::AssetV1);
    assert_eq!(fetched_core_asset.base.name, name);
    assert_eq!(fetched_core_asset.base.uri, uri);
    assert_eq!(fetched_core_asset.base.owner, alice.pubkey());
    assert_eq!(
        fetched_core_asset.base.update_authority,
        UpdateAuthority::Collection(collection)
    );
    assert!(fetched_core_asset.base.seq.is_none());

    let data = prg_test_ctx
        .banks_client
        .get_account(core_record)
        .await
        .unwrap()
        .unwrap()
        .data;
    let fetched_core_record = CoreRecord::deserialize(&mut data.as_slice()).unwrap();
    assert_eq!(fetched_core_record.core_asset, core_asset);
    assert_eq!(fetched_core_record.name_account, name_key);
    assert!(matches!(fetched_core_record.tag, Tag::ActiveCoreRecord));
    assert_eq!(fetched_core_record.owner, alice.pubkey());

    assert!(plugin_list.attributes.is_none());
    assert!(plugin_list.freeze_delegate.is_none());
    assert!(plugin_list.burn_delegate.is_none());
    assert!(plugin_list.transfer_delegate.is_none());
    assert!(plugin_list.update_delegate.is_none());
    assert!(plugin_list.edition.is_none());
    assert!(plugin_list.permanent_burn_delegate.is_none());

    assert_eq!(
        plugin_list
            .permanent_freeze_delegate
            .clone()
            .unwrap()
            .base
            .authority
            .address
            .unwrap(),
        name_tokenizer::central_state::KEY
    );
    assert_eq!(
        plugin_list
            .permanent_freeze_delegate
            .clone()
            .unwrap()
            .permanent_freeze_delegate
            .frozen,
        false
    );
    assert_eq!(
        plugin_list
            .permanent_transfer_delegate
            .clone()
            .unwrap()
            .base
            .authority
            .address
            .unwrap(),
        name_tokenizer::central_state::KEY
    );

    ///////////////////////////////////////////////////
    // Withdraw tokens Core
    // - (1) We mint tokens to the `CoreAsset` escrow + transfer some SOL
    // - (2) Alice tries to withdraw these tokens
    ///////////////////////////////////////////////////

    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &core_record,
        &usdc_mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let amount = 10_000_000_000;
    let escrow_pda_ata = get_associated_token_address(&core_record, &usdc_mint);
    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &usdc_mint,
        &escrow_pda_ata,
        &mint_authority.pubkey(),
        &[],
        10_000_000_000,
    )
    .unwrap();
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&mint_authority])
        .await
        .unwrap();

    let extra_lamports = 33_333_333;
    let ix =
        system_instruction::transfer(&prg_test_ctx.payer.pubkey(), &core_record, extra_lamports);
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let alice_ata = get_associated_token_address(&alice.pubkey(), &usdc_mint);
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &alice.pubkey(),
        &usdc_mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let ix = withdraw_tokens_core(
        withdraw_tokens_core::Accounts {
            core_asset: &core_asset,
            core_asset_owner: &alice.pubkey(),
            core_record: &core_record,
            token_destination: &alice_ata,
            token_source: &escrow_pda_ata,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
        },
        withdraw_tokens_core::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    let fetched_ata = prg_test_ctx
        .banks_client
        .get_account(escrow_pda_ata)
        .await
        .unwrap()
        .unwrap()
        .data;

    let des = spl_token::state::Account::unpack(&fetched_ata).unwrap();
    assert_eq!(des.amount, 0);

    let fetched_ata = prg_test_ctx
        .banks_client
        .get_account(alice_ata)
        .await
        .unwrap()
        .unwrap()
        .data;

    let des = spl_token::state::Account::unpack(&fetched_ata).unwrap();
    assert_eq!(des.amount, amount);

    let alice_balance = prg_test_ctx
        .banks_client
        .get_balance(alice.pubkey())
        .await
        .unwrap();
    assert_eq!(alice_balance, extra_lamports);
}