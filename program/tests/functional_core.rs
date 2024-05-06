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
    instruction::{
        create_collection_core, create_nft_core, redeem_nft_core, renew, withdraw_tokens_core,
    },
    state::CoreRecord,
};

use crate::common::utils::{mint_bootstrap, sign_send_instructions};

#[tokio::test]
async fn test_mpl_core() {
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

    program_test.add_account(
        alice.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );
    program_test.add_account(
        bob.pubkey(),
        Account {
            lamports: 100_000_000_000,
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
    // Create NFT (Core Asset)
    ///////////////////////////////////////////////////
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
            uri: "https://...".to_owned(),
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

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

    ///////////////////////////////////////////////////
    // Recreate NFT (Core Asset)
    ///////////////////////////////////////////////////
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
            uri: "https://...".to_owned(),
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    ///////////////////////////////////////////////////
    // Withdraw tokens Core
    // - (1) We mint tokens to the `CoreAsset` escrow
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

    ///////////////////////////////////////////////////
    // Transfer Core Asset to Bob + Withdraw tokens Core
    // - (1) Transfer to Bob
    // - (2) Bob withdraws the tokens
    ///////////////////////////////////////////////////

    let ix = mpl_core::instructions::TransferV1 {
        collection: Some(collection),
        asset: core_asset,
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
            core_asset: &core_asset,
            core_asset_owner: &bob.pubkey(),
            core_record: &core_record,
            token_destination: &alice_ata,
            token_source: &escrow_pda_ata,
            system_program: &system_program::ID,
            spl_token_program: &spl_token::ID,
        },
        withdraw_tokens_core::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    ///////////////////////////////////////////////////
    // Renew and transfer back to SNS Registrar
    ///////////////////////////////////////////////////

    let renew_authority = Keypair::new();
    let ix = renew(
        renew::Accounts {
            central_state: &name_tokenizer::central_state::KEY,
            collection: &collection,
            core_asset: &core_asset,
            core_record: &core_record,
            renew_authority: &renew_authority.pubkey(),
            fee_payer: &prg_test_ctx.payer.pubkey(),
            name_account: &name_key,
            mpl_core_program: &mpl_core::ID,
            spl_name_service_program: &spl_name_service::ID,
            system_program: &system_program::ID,
        },
        renew::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&renew_authority])
        .await
        .unwrap();
}
