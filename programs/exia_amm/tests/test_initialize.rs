use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{
            instruction::Instruction,
            system_instruction,
            system_program,
            rent::Rent,
        },
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn send_tx(svm: &mut LiteSVM, payer: &Keypair, ixs: Vec<Instruction>, signers: &[&Keypair]) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx).expect("Transaction failed");
}

struct PoolFixture {
    svm: LiteSVM,
    payer: Keypair,
    token_a_mint: Keypair,
    token_b_mint: Keypair,
    pool_state_pda: Pubkey,
    vault_a_pda: Pubkey,
    vault_b_pda: Pubkey,
    lp_mint_pda: Pubkey,
}

fn setup_initialized_pool() -> PoolFixture {
    let program_id = exia_amm::id();
    let payer = Keypair::new();
    let token_a_mint = Keypair::new();
    let token_b_mint = Keypair::new();

    let (pool_state_pda, _) = Pubkey::find_program_address(
        &[b"pool", token_a_mint.pubkey().as_ref(), token_b_mint.pubkey().as_ref()],
        &program_id,
    );
    let (vault_a_pda, _) = Pubkey::find_program_address(&[b"vault_a", pool_state_pda.as_ref()], &program_id);
    let (vault_b_pda, _) = Pubkey::find_program_address(&[b"vault_b", pool_state_pda.as_ref()], &program_id);
    let (lp_mint_pda, _) = Pubkey::find_program_address(&[b"lp_mint", pool_state_pda.as_ref()], &program_id);

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/exia_amm.so"));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();

    let mint_space = anchor_spl::token::Mint::LEN as u64;
    let mint_rent = Rent::default().minimum_balance(mint_space as usize);

    send_tx(&mut svm, &payer, vec![
        system_instruction::create_account(&payer.pubkey(), &token_a_mint.pubkey(), mint_rent, mint_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_mint(&anchor_spl::token::ID, &token_a_mint.pubkey(), &payer.pubkey(), None, 6).unwrap(),
        system_instruction::create_account(&payer.pubkey(), &token_b_mint.pubkey(), mint_rent, mint_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_mint(&anchor_spl::token::ID, &token_b_mint.pubkey(), &payer.pubkey(), None, 6).unwrap(),
    ], &[&payer, &token_a_mint, &token_b_mint]);

    send_tx(&mut svm, &payer, vec![
        Instruction::new_with_bytes(
            program_id,
            &exia_amm::instruction::InitializePool { lp_fee_bps: 25, protocol_fee_bps: 5 }.data(),
            exia_amm::accounts::InitializePool {
                payer: payer.pubkey(),
                pool_state: pool_state_pda,
                token_a_mint: token_a_mint.pubkey(),
                token_b_mint: token_b_mint.pubkey(),
                vault_a: vault_a_pda,
                vault_b: vault_b_pda,
                lp_mint: lp_mint_pda,
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
                rent: "SysvarRent111111111111111111111111111111111".parse().unwrap(),
            }.to_account_metas(None),
        )
    ], &[&payer]);

    PoolFixture { svm, payer, token_a_mint, token_b_mint, pool_state_pda, vault_a_pda, vault_b_pda, lp_mint_pda }
}

#[test]
fn test_initialize_pool() {
    let f = setup_initialized_pool();

    let pool_account = f.svm.get_account(&f.pool_state_pda).unwrap();
    let mut data: &[u8] = &pool_account.data;
    let pool_state = exia_amm::state::PoolState::try_deserialize(&mut data).unwrap();

    assert_eq!(pool_state.token_a_mint, f.token_a_mint.pubkey());
    assert_eq!(pool_state.token_b_mint, f.token_b_mint.pubkey());
    assert_eq!(pool_state.token_a_vault, f.vault_a_pda);
    assert_eq!(pool_state.token_b_vault, f.vault_b_pda);
    assert_eq!(pool_state.lp_mint, f.lp_mint_pda);
    assert_eq!(pool_state.lp_fee_bps, 25);
    assert_eq!(pool_state.protocol_fee_bps, 5);
    assert_eq!(pool_state.k_last, 0);

    println!("SUCCESS! Pool initialized at: {:?}", f.pool_state_pda);
}

#[test]
fn test_add_liquidity() {
    let mut f = setup_initialized_pool();
    let program_id = exia_amm::id();

    let token_account_space = anchor_spl::token::TokenAccount::LEN as u64;
    let token_account_rent = Rent::default().minimum_balance(token_account_space as usize);

    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    let user_lp_token = Keypair::new();

    send_tx(&mut f.svm, &f.payer, vec![
        system_instruction::create_account(&f.payer.pubkey(), &user_token_a.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &user_token_a.pubkey(), &f.token_a_mint.pubkey(), &f.payer.pubkey()).unwrap(),
        system_instruction::create_account(&f.payer.pubkey(), &user_token_b.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &user_token_b.pubkey(), &f.token_b_mint.pubkey(), &f.payer.pubkey()).unwrap(),
        system_instruction::create_account(&f.payer.pubkey(), &user_lp_token.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &user_lp_token.pubkey(), &f.lp_mint_pda, &f.payer.pubkey()).unwrap(),
    ], &[&f.payer.insecure_clone(), &user_token_a, &user_token_b, &user_lp_token]);

    let amount_a: u64 = 10_000_000;
    let amount_b: u64 = 20_000_000;

    send_tx(&mut f.svm, &f.payer, vec![
        anchor_spl::token::spl_token::instruction::mint_to(&anchor_spl::token::ID, &f.token_a_mint.pubkey(), &user_token_a.pubkey(), &f.payer.pubkey(), &[], amount_a).unwrap(),
        anchor_spl::token::spl_token::instruction::mint_to(&anchor_spl::token::ID, &f.token_b_mint.pubkey(), &user_token_b.pubkey(), &f.payer.pubkey(), &[], amount_b).unwrap(),
    ], &[&f.payer.insecure_clone()]);

    let add_liq_ix = Instruction::new_with_bytes(
        program_id,
        &exia_amm::instruction::AddLiquidity { amount_a, amount_b }.data(),
        exia_amm::accounts::AddLiquidity {
            user: f.payer.pubkey(),
            pool_state: f.pool_state_pda,
            user_token_a: user_token_a.pubkey(),
            user_token_b: user_token_b.pubkey(),
            user_lp_token: user_lp_token.pubkey(),
            vault_a: f.vault_a_pda,
            vault_b: f.vault_b_pda,
            lp_mint: f.lp_mint_pda,
            token_program: anchor_spl::token::ID,
            system_program: system_program::ID,
        }.to_account_metas(None),
    );

    let blockhash = f.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[add_liq_ix], Some(&f.payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&f.payer.insecure_clone()]).unwrap();
    let res = f.svm.send_transaction(tx);
    assert!(res.is_ok(), "add_liquidity failed: {:?}", res);

    let vault_a_data = f.svm.get_account(&f.vault_a_pda).unwrap().data;
    let vault_a_state = anchor_spl::token::TokenAccount::try_deserialize(&mut vault_a_data.as_slice()).unwrap();
    assert_eq!(vault_a_state.amount, amount_a);

    let vault_b_data = f.svm.get_account(&f.vault_b_pda).unwrap().data;
    let vault_b_state = anchor_spl::token::TokenAccount::try_deserialize(&mut vault_b_data.as_slice()).unwrap();
    assert_eq!(vault_b_state.amount, amount_b);

    let user_lp_data = f.svm.get_account(&user_lp_token.pubkey()).unwrap().data;
    let user_lp_state = anchor_spl::token::TokenAccount::try_deserialize(&mut user_lp_data.as_slice()).unwrap();
    assert!(user_lp_state.amount > 0, "User should have received LP tokens");

    let pool_data = f.svm.get_account(&f.pool_state_pda).unwrap().data;
    let pool_state = exia_amm::state::PoolState::try_deserialize(&mut pool_data.as_slice()).unwrap();
    assert_eq!(pool_state.k_last, amount_a as u128 * amount_b as u128);

    println!("SUCCESS! add_liquidity passed. LP tokens minted: {}", user_lp_state.amount);
}
