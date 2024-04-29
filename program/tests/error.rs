use {
    borsh::BorshSerialize,
    itertools::Itertools,
    name_tokenizer::{
        entrypoint::process_instruction,
        state::{METADATA_SIGNER, ROOT_DOMAIN_ACCOUNT},
    },
    solana_program::{hash::hashv, pubkey::Pubkey, system_program},
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{
        account::Account,
        signer::{keypair::Keypair, Signer},
    },
    spl_associated_token_account::{
        get_associated_token_address, instruction::create_associated_token_account,
    },
    spl_name_service::state::{get_seeds_and_key, HASH_PREFIX},
};

pub mod common;

use mpl_core::instructions::TransferV1InstructionArgs;
use name_tokenizer::{
    instruction::{create_collection_core, create_nft_core, redeem_nft_core, withdraw_tokens_core},
    state::CoreRecord,
};

use crate::common::utils::{mint_bootstrap, sign_send_instructions};

#[tokio::test]
async fn test_error_mpl_core() {
    // Create program and test environment
    let alice = Keypair::new();
    let bob = Keypair::new();
    let mint_authority = Keypair::new();

    let mut program_test = ProgramTest::new(
        "name_tokenizer",
        name_tokenizer::ID,
        processor!(process_instruction),
    );
    program_test.add_program("spl_name_service", spl_name_service::ID, None);
    program_test.add_program("core", mpl_core::ID, None);

    // Create domain names
    let name = "alice_domain";
    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec();

    let (alice_domain, _) = get_seeds_and_key(
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
        alice_domain,
        Account {
            lamports: 1_000_000,
            data: name_domain_data,
            owner: spl_name_service::id(),
            ..Account::default()
        },
    );

    let name = "bob_domain";
    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec();

    let (bob_domain, _) = get_seeds_and_key(
        &spl_name_service::ID,
        hashed_name,
        None,
        Some(&ROOT_DOMAIN_ACCOUNT),
    );

    let name_domain_data = [
        spl_name_service::state::NameRecordHeader {
            parent_name: ROOT_DOMAIN_ACCOUNT,
            owner: bob.pubkey(),
            class: Pubkey::default(),
        }
        .try_to_vec()
        .unwrap(),
        vec![0; 1000],
    ]
    .concat();

    program_test.add_account(
        bob_domain,
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

    ///////////////////////////////////////////////////
    // Alice & Bob tokenize their domains
    ///////////////////////////////////////////////////
    let (alice_core_asset, _) = name_tokenizer::utils::get_core_nft_key(&alice_domain);
    let (alice_core_record, _) = CoreRecord::find_key(&alice_domain, &name_tokenizer::ID);
    let ix = create_nft_core(
        create_nft_core::Accounts {
            core_asset: &alice_core_asset,
            name_account: &alice_domain,
            collection: &collection,
            core_record: &alice_core_record,
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
            uri: "https://...".to_owned(),
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    let (bob_core_asset, _) = name_tokenizer::utils::get_core_nft_key(&bob_domain);
    let (bob_core_record, _) = CoreRecord::find_key(&bob_domain, &name_tokenizer::ID);
    let ix = create_nft_core(
        create_nft_core::Accounts {
            core_asset: &bob_core_asset,
            name_account: &bob_domain,
            collection: &collection,
            core_record: &bob_core_record,
            name_owner: &bob.pubkey(),
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
            uri: "https://...".to_owned(),
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    ///////////////////////////////////////////////////
    // 1. Alice tries to redeem Bob domains
    // 2. Alice tries to withdraw tokens from Bob domains
    // 3. Alice transfer core asset to Bob and tries to withdraw tokens
    ///////////////////////////////////////////////////

    ///////////////////////////////////////////////////
    // 1. Alice tries to redeem Bob domains
    ///////////////////////////////////////////////////

    let core_assets = vec![bob_core_asset, alice_core_asset];
    let core_records = vec![alice_core_record, bob_core_record];
    let domains = vec![alice_domain, bob_domain];
    let test_cases = vec![core_assets.clone(), core_records.clone(), domains];
    let excluded = vec![&alice_core_asset, &alice_core_record, &alice_domain];

    let mut combinations = test_cases.iter().multi_cartesian_product();

    while let Some(current) = combinations.next() {
        if current == excluded {
            continue;
        }
        let ix = redeem_nft_core(
            redeem_nft_core::Accounts {
                core_asset: &current.get(0).unwrap(),
                core_asset_owner: &alice.pubkey(),
                collection: &collection,
                core_record: &current.get(1).unwrap(),
                name_account: &current.get(2).unwrap(),
                central_state: &name_tokenizer::central_state::KEY,
                mpl_core_program: &mpl_core::ID,
                spl_name_service_program: &spl_name_service::ID,
                system_program: &system_program::ID,
            },
            redeem_nft_core::Params {},
        );
        let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
        assert!(res.is_err());
    }

    ///////////////////////////////////////////////////
    // 2. Alice tries to withdraw tokens from Bob domains
    ///////////////////////////////////////////////////

    // Prepare ATAs
    let alice_ata = get_associated_token_address(&alice.pubkey(), &usdc_mint);
    let bob_core_record_ata = get_associated_token_address(&&bob_core_record, &usdc_mint);
    let alice_core_record_ata = get_associated_token_address(&&alice_core_record, &usdc_mint);

    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &bob_core_record,
        &usdc_mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &alice_core_record,
        &usdc_mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &bob.pubkey(),
        &usdc_mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let ix = create_associated_token_account(
        &prg_test_ctx.payer.pubkey(),
        &alice.pubkey(),
        &usdc_mint,
        &spl_token::ID,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let token_sources = vec![alice_core_record_ata, bob_core_record_ata];

    let test_cases = vec![core_assets, core_records, token_sources];
    let excluded = vec![
        &alice_core_asset,
        &alice_core_record,
        &alice_core_record_ata,
    ];

    let mut combinations = test_cases.iter().multi_cartesian_product();

    while let Some(current) = combinations.next() {
        if current == excluded {
            continue;
        }
        let ix = withdraw_tokens_core(
            withdraw_tokens_core::Accounts {
                core_asset: &current.get(0).unwrap(),
                core_asset_owner: &alice.pubkey(),
                core_record: &current.get(1).unwrap(),
                token_destination: &alice_ata,
                token_source: &current.get(2).unwrap(),
                system_program: &system_program::ID,
                spl_token_program: &spl_token::ID,
            },
            withdraw_tokens_core::Params {},
        );
        let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
        assert!(res.is_err())
    }

    ///////////////////////////////////////////////////
    // 3. Alice transfer core asset to Bob and tries to withdraw tokens
    ///////////////////////////////////////////////////

    // Transfer core assets to Bob
    let ix = mpl_core::instructions::TransferV1 {
        collection: Some(collection),
        asset: alice_core_asset,
        payer: prg_test_ctx.payer.pubkey(),
        authority: Some(alice.pubkey()),
        new_owner: bob.pubkey(),
        system_program: None,
        log_wrapper: None,
    };

    sign_send_instructions(
        &mut prg_test_ctx,
        vec![ix.instruction(TransferV1InstructionArgs {
            compression_proof: None,
        })],
        vec![&alice],
    )
    .await
    .unwrap();

    let ix = withdraw_tokens_core(
        withdraw_tokens_core::Accounts {
            core_asset: &alice_core_asset,
            core_asset_owner: &alice.pubkey(),
            core_record: &alice_core_record,
            token_destination: &alice_ata,
            token_source: &alice_core_record_ata,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
        },
        withdraw_tokens_core::Params {},
    );
    let res = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice]).await;
    assert!(res.is_err())
}
