mod test_handler;

use {
    anchor_lang::solana_program::instruction::Instruction,
    anchor_spl::token_2022::spl_token_2022::{
        extension::{BaseStateWithExtensions, StateWithExtensions},
        state::Mint as MintState,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::Message,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

pub(crate) fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    ixs: Vec<Instruction>,
) {
    let blockhash = svm.latest_blockhash();
    let mut all_signers: Vec<&Keypair> = vec![payer];
    all_signers.extend_from_slice(extra_signers);
    let msg = Message::new(&ixs, Some(&payer.pubkey()));
    let tx = Transaction::new(&all_signers, msg, blockhash);
    svm.send_transaction(tx).expect("transaction failed");
}

pub(crate) fn new_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    let binary_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/deploy/token22_ct.so"
    );
    svm.add_program_from_file(token22_ct::ID, binary_path)
        .expect("program .so not found – run `cargo build-sbf` before `cargo test`");
    svm
}

#[test]
fn test_initialize_mint_v1_extensions() {
    let mut svm = new_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mint = test_handler::initialize::init_mint(&mut svm, &payer);
    let data = test_handler::initialize::mint_data(&svm, &mint);

    let mint_state = StateWithExtensions::<MintState>::unpack(&data)
        .expect("must parse via StateWithExtensions");

    let ext_types = mint_state.get_extension_types().unwrap();

    for required in &test_handler::initialize::fixed_extensions() {
        assert!(
            ext_types.contains(required),
            "v1 mint missing extension {required:?}"
        );
    }
}

#[test]
fn test_transfer_fee_via_program() {
    let mut svm = new_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mint = test_handler::initialize::init_mint(&mut svm, &payer);
    let alice = test_handler::transfer_fee::create_token_account(&mut svm, &payer, &mint, &payer);
    let bob = test_handler::transfer_fee::create_token_account(&mut svm, &payer, &mint, &payer);

    let mint_amount = 1_000_000u64;
    test_handler::transfer_fee::mint_to(&mut svm, &payer, &mint, &alice, mint_amount);

    let (alice_pre, _) = test_handler::transfer_fee::balances(&svm, &alice);
    assert_eq!(alice_pre, mint_amount);

    let transfer_amount = 100_000u64;
    test_handler::transfer_fee::transfer_via_program(
        &mut svm,
        &payer,
        &payer,
        &alice,
        &bob,
        &mint,
        transfer_amount,
    );

    let (alice_post, _) = test_handler::transfer_fee::balances(&svm, &alice);
    let (bob_post, bob_withheld) = test_handler::transfer_fee::balances(&svm, &bob);

    assert_eq!(alice_post, mint_amount - transfer_amount);
    let expected_fee = (transfer_amount as u128 * 100 / 10_000) as u64;
    assert_eq!(bob_post + bob_withheld, transfer_amount);
    assert_eq!(bob_withheld, expected_fee, "fee must be 1% (100 bps)");
}

#[test]
fn test_new_accounts_frozen_by_default() {
    let mut svm = new_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mint = test_handler::initialize::init_mint(&mut svm, &payer);
    let frozen_account =
        test_handler::unfreeze::create_frozen_account(&mut svm, &payer, &mint, &payer);

    assert!(
        !test_handler::transfer_fee::is_unfrozen(&svm, &frozen_account),
        "new account should be frozen by default (DefaultAccountState = Frozen)"
    );
}

#[test]
fn test_unfreeze_after_kyc() {
    let mut svm = new_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mint = test_handler::initialize::init_mint(&mut svm, &payer);
    let account = test_handler::unfreeze::create_frozen_account(&mut svm, &payer, &mint, &payer);

    assert!(
        !test_handler::transfer_fee::is_unfrozen(&svm, &account),
        "account should start frozen"
    );

    test_handler::unfreeze::unfreeze_via_program(&mut svm, &payer, &payer, &account, &mint);

    assert!(
        test_handler::transfer_fee::is_unfrozen(&svm, &account),
        "account should be unfrozen after KYC approval"
    );
}

#[test]
fn test_transfer_after_thaw() {
    let mut svm = new_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mint = test_handler::initialize::init_mint(&mut svm, &payer);
    let alice = test_handler::transfer_fee::create_token_account(&mut svm, &payer, &mint, &payer);
    let bob = test_handler::unfreeze::create_frozen_account(&mut svm, &payer, &mint, &payer);

    test_handler::transfer_fee::mint_to(&mut svm, &payer, &mint, &alice, 1_000_000);
    test_handler::unfreeze::unfreeze_via_program(&mut svm, &payer, &payer, &bob, &mint);

    test_handler::transfer_fee::transfer_via_program(
        &mut svm, &payer, &payer, &alice, &bob, &mint, 50_000,
    );

    let (bob_bal, _) = test_handler::transfer_fee::balances(&svm, &bob);
    assert!(bob_bal > 0, "bob received tokens after being unfrozen");
}

#[test]
fn test_mint_v2_extensions() {
    let mut svm = new_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mint = test_handler::initialize_v2::init_mint_v2(&mut svm, &payer);
    let data = test_handler::initialize_v2::mint_data_v2(&svm, &mint);

    let mint_state = StateWithExtensions::<MintState>::unpack(&data)
        .expect("must parse via StateWithExtensions");

    let ext_types = mint_state.get_extension_types().unwrap();

    for required in &test_handler::initialize_v2::fixed_v2_extensions() {
        assert!(
            ext_types.contains(required),
            "v2 mint missing extension {required:?}"
        );
    }
}

#[test]
fn test_state_read_via_state_with_extensions() {
    let mut svm = new_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mint = test_handler::initialize::init_mint(&mut svm, &payer);
    let data = svm
        .get_account(&mint.pubkey())
        .expect("mint must exist")
        .data
        .clone();

    let parsed = StateWithExtensions::<MintState>::unpack(&data)
        .expect("StateWithExtensions unpack must succeed");
    assert_eq!(parsed.base.decimals, token22_ct::DECIMALS);
    assert!(parsed.base.is_initialized, "mint must be initialized");
}
