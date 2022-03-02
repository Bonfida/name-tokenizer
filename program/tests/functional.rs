use borsh::BorshSerialize;
use solana_program::{pubkey, pubkey::Pubkey};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::account::Account;
use solana_sdk::signer::{keypair::Keypair, Signer};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
pub mod common;
use crate::common::utils::{mint_bootstrap, sign_send_instructions};
use name_offers::{
    entrypoint::process_instruction,
    instruction::{
        accept_offer, buy_fixed_price, cancel_fixed_price, cancel_offer, make_fixed_price,
        make_offer, register_favourite,
    },
    state::{FavouriteDomain, FixedPriceOffer, Offer, FEE_OWNER, ROOT_DOMAIN_ACCOUNT},
};
use solana_program::{system_instruction, system_program, sysvar};

#[tokio::test]
async fn test_offer() {
    // Create program and test environment
    let program_id = pubkey!("hxrotrKwueSFofXvCmCpYyKMjn1BhmwKtPxA1nLcv8m");
    let seller = Keypair::new();
    let buyer = Keypair::new();
    let mint_authority = Keypair::new();
    let offer_amount: u64 = 10_000_000;

    let mut program_test =
        ProgramTest::new("name_offers", program_id, processor!(process_instruction));
    program_test.add_program("spl_name_service", spl_name_service::ID, None);

    let root_domain_data = spl_name_service::state::NameRecordHeader {
        parent_name: ROOT_DOMAIN_ACCOUNT,
        owner: seller.pubkey(),
        class: Pubkey::default(),
    }
    .try_to_vec()
    .unwrap();

    let name_key = Keypair::new().pubkey();

    program_test.add_account(
        name_key,
        Account {
            lamports: 1_000_000,
            data: root_domain_data,
            owner: spl_name_service::id(),
            ..Account::default()
        },
    );

    //
    // Create mint
    //
    let (mint, _) = mint_bootstrap(None, 6, &mut program_test, &mint_authority.pubkey());

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    // Send some SOL
    let ix = system_instruction::transfer(
        &prg_test_ctx.payer.pubkey(),
        &buyer.pubkey(),
        1_000_000_000_000 * 10,
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    ////
    // Mint tokens in buyer account
    ////
    let ix = create_associated_token_account(&prg_test_ctx.payer.pubkey(), &buyer.pubkey(), &mint);
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let buyer_ata = get_associated_token_address(&buyer.pubkey(), &mint);
    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint,
        &buyer_ata,
        &mint_authority.pubkey(),
        &[],
        offer_amount,
    )
    .unwrap();

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&mint_authority])
        .await
        .unwrap();

    ////
    // Make fee account
    ////
    let ix = create_associated_token_account(&prg_test_ctx.payer.pubkey(), &FEE_OWNER, &mint);
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let fee_account = get_associated_token_address(&FEE_OWNER, &mint);

    // Make an offer
    let (offer_key, _) = Offer::find_key(&buyer.pubkey(), &name_key, &mint, &program_id);

    let (escrow_account, _) = Pubkey::find_program_address(&[&offer_key.to_bytes()], &program_id);
    let ix = make_offer(
        make_offer::Accounts {
            owner: &buyer.pubkey(),
            quote_mint: &mint,
            token_source: &buyer_ata,
            escrow_token_account: &escrow_account,
            offer_account: &offer_key,
            system_program: &system_program::ID,
            rent_account: &sysvar::rent::ID,
            spl_token_program: &spl_token::ID,
        },
        make_offer::Params {
            amount: offer_amount,
            name_account: name_key,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&buyer])
        .await
        .unwrap();

    // Cancel offer
    let ix = cancel_offer(
        cancel_offer::Accounts {
            owner: &buyer.pubkey(),
            token_destination: &buyer_ata,
            escrow_account: &escrow_account,
            offer_account: &offer_key,
            spl_token_program: &spl_token::ID,
        },
        cancel_offer::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&buyer])
        .await
        .unwrap();

    // Make new offer
    let ix = make_offer(
        make_offer::Accounts {
            owner: &buyer.pubkey(),
            quote_mint: &mint,
            token_source: &buyer_ata,
            escrow_token_account: &escrow_account,
            offer_account: &offer_key,
            system_program: &system_program::ID,
            rent_account: &sysvar::rent::ID,
            spl_token_program: &spl_token::ID,
        },
        make_offer::Params {
            amount: offer_amount - 1,
            name_account: name_key,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&buyer])
        .await
        .unwrap();

    // Accept offer
    let ix = create_associated_token_account(&prg_test_ctx.payer.pubkey(), &seller.pubkey(), &mint);
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let seller_ata = get_associated_token_address(&seller.pubkey(), &mint);

    let ix = accept_offer(
        accept_offer::Accounts {
            offer_beneficiary: &seller.pubkey(),
            name_account: &name_key,
            destination_token_account: &seller_ata,
            escrow_token_account: &escrow_account,
            offer_account: &offer_key,
            spl_token_program: &spl_token::ID,
            name_service_program: &spl_name_service::ID,
            fee_account: &fee_account,
        },
        accept_offer::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&seller])
        .await
        .unwrap();

    // Register favourite domain
    let favourite_key = FavouriteDomain::find_key(&buyer.pubkey(), &program_id).0;
    let ix = register_favourite(
        register_favourite::Accounts {
            name_account: &name_key,
            favourite_account: &favourite_key,
            owner: &buyer.pubkey(),
            system_program: &system_program::ID,
        },
        register_favourite::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&buyer])
        .await
        .unwrap();

    // Fixed price offer

    let (fixed_price_key, _) =
        FixedPriceOffer::find_key(&buyer.pubkey(), &name_key, &mint, &program_id);
    let ix = make_fixed_price(
        make_fixed_price::Accounts {
            fixed_price_offer: &fixed_price_key,
            fixed_price_offer_owner: &buyer.pubkey(),
            name_account: &name_key,
            token_destination: &buyer_ata,
            name_service_program: &spl_name_service::ID,
            system_program: &system_program::ID,
        },
        make_fixed_price::Params {
            amount: offer_amount,
            quote_mint: mint,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&buyer])
        .await
        .unwrap();

    // Cancel fixed price
    let ix = cancel_fixed_price(
        cancel_fixed_price::Accounts {
            fixed_price_offer: &fixed_price_key,
            fixed_price_offer_owner: &buyer.pubkey(),
            name_account: &name_key,
            name_service_program: &spl_name_service::ID,
            system_program: &system_program::ID,
        },
        cancel_fixed_price::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&buyer])
        .await
        .unwrap();

    // Make new fixed price offer
    let ix = make_fixed_price(
        make_fixed_price::Accounts {
            fixed_price_offer: &fixed_price_key,
            fixed_price_offer_owner: &buyer.pubkey(),
            name_account: &name_key,
            token_destination: &buyer_ata,
            name_service_program: &spl_name_service::ID,
            system_program: &system_program::ID,
        },
        make_fixed_price::Params {
            amount: offer_amount / 2,
            quote_mint: mint,
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&buyer])
        .await
        .unwrap();

    // Buy fixed price

    let ix = buy_fixed_price(
        buy_fixed_price::Accounts {
            fixed_price_offer: &fixed_price_key,
            buyer: &seller.pubkey(),
            name_account: &name_key,
            token_destination: &buyer_ata,
            token_source: &seller_ata,
            fee_account: &fee_account,
            spl_token_program: &spl_token::ID,
            name_service_program: &spl_name_service::ID,
        },
        buy_fixed_price::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&seller])
        .await
        .unwrap();
}
