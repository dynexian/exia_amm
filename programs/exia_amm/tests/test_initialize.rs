use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_initialize_pool() {
    let program_id = exia_amm::id();
    let payer = Keypair::new();

    let token_a_mint = Keypair::new();
    let token_b_mint = Keypair::new();

    let (pool_state_pda, _bump) = Pubkey::find_program_address(
        &[
            b"pool",
            token_a_mint.pubkey().as_ref(),
            token_b_mint.pubkey().as_ref(),
        ],
        &program_id,
    );

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/exia_amm.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &exia_amm::instruction::InitializePool {
            lp_fee_bps: 25,
            protocol_fee_bps: 5,
        }
        .data(),
        exia_amm::accounts::InitializePool {
            payer: payer.pubkey(),
            pool_state: pool_state_pda,
            token_a_mint: token_a_mint.pubkey(),
            token_b_mint: token_b_mint.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "Transaction failed: {:?}", res);

    let pool_account = svm.get_account(&pool_state_pda).unwrap();
    let mut data: &[u8] = &pool_account.data;
    let pool_state = exia_amm::state::PoolState::try_deserialize(&mut data).unwrap();

    assert_eq!(pool_state.token_a_mint, token_a_mint.pubkey());
    assert_eq!(pool_state.token_b_mint, token_b_mint.pubkey());
    assert_eq!(pool_state.lp_fee_bps, 25);
    assert_eq!(pool_state.protocol_fee_bps, 5);

    println!("Pool initialized successfully at: {:?}", pool_state_pda);
}
