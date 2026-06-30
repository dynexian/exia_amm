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
            &exia_amm::instruction::InitializePool { lp_fee_bps: 25, protocol_fee_bps: 5, treasury_wallet: payer.pubkey() }.data(),
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

#[test]
fn test_swap_a_to_b() {
    let mut f = setup_initialized_pool();
    let program_id = exia_amm::id();

    let token_account_space = anchor_spl::token::TokenAccount::LEN as u64;
    let token_account_rent = Rent::default().minimum_balance(token_account_space as usize);

    // --- Create treasury token account for Token A (receives protocol fees) ---
    let treasury_token_a = Keypair::new();
    send_tx(&mut f.svm, &f.payer, vec![
        system_instruction::create_account(&f.payer.pubkey(), &treasury_token_a.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &treasury_token_a.pubkey(), &f.token_a_mint.pubkey(), &f.payer.pubkey()).unwrap(),
    ], &[&f.payer.insecure_clone(), &treasury_token_a]);

    // --- Update pool_state.treasury_wallet to point to this token account ---
    // We need to re-initialize with the correct treasury — for the test we'll
    // directly patch pool_state via svm.set_account after deserializing,
    // OR simply re-create the pool with treasury set correctly from the start.
    // Easiest: create a fresh pool with treasury_token_a as the wallet.
    let payer2 = Keypair::new();
    let token_a_mint2 = Keypair::new();
    let token_b_mint2 = Keypair::new();
    f.svm.airdrop(&payer2.pubkey(), 100_000_000_000).unwrap();

    let (pool2_pda, _) = Pubkey::find_program_address(
        &[b"pool", token_a_mint2.pubkey().as_ref(), token_b_mint2.pubkey().as_ref()],
        &program_id,
    );
    let (vault_a2, _) = Pubkey::find_program_address(&[b"vault_a", pool2_pda.as_ref()], &program_id);
    let (vault_b2, _) = Pubkey::find_program_address(&[b"vault_b", pool2_pda.as_ref()], &program_id);
    let (lp_mint2, _) = Pubkey::find_program_address(&[b"lp_mint", pool2_pda.as_ref()], &program_id);

    let mint_space = anchor_spl::token::Mint::LEN as u64;
    let mint_rent = Rent::default().minimum_balance(mint_space as usize);

    // Create mints
    send_tx(&mut f.svm, &payer2, vec![
        system_instruction::create_account(&payer2.pubkey(), &token_a_mint2.pubkey(), mint_rent, mint_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_mint(&anchor_spl::token::ID, &token_a_mint2.pubkey(), &payer2.pubkey(), None, 6).unwrap(),
        system_instruction::create_account(&payer2.pubkey(), &token_b_mint2.pubkey(), mint_rent, mint_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_mint(&anchor_spl::token::ID, &token_b_mint2.pubkey(), &payer2.pubkey(), None, 6).unwrap(),
    ], &[&payer2, &token_a_mint2, &token_b_mint2]);

    // Create treasury token account for this pool's Token A
    let treasury = Keypair::new();
    send_tx(&mut f.svm, &payer2, vec![
        system_instruction::create_account(&payer2.pubkey(), &treasury.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &treasury.pubkey(), &token_a_mint2.pubkey(), &payer2.pubkey()).unwrap(),
    ], &[&payer2, &treasury]);

    // Initialize pool with treasury set
    send_tx(&mut f.svm, &payer2, vec![
        Instruction::new_with_bytes(
            program_id,
            &exia_amm::instruction::InitializePool {
                lp_fee_bps: 25,
                protocol_fee_bps: 5,
                treasury_wallet: treasury.pubkey(),
            }.data(),
            exia_amm::accounts::InitializePool {
                payer: payer2.pubkey(),
                pool_state: pool2_pda,
                token_a_mint: token_a_mint2.pubkey(),
                token_b_mint: token_b_mint2.pubkey(),
                vault_a: vault_a2,
                vault_b: vault_b2,
                lp_mint: lp_mint2,
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
                rent: "SysvarRent111111111111111111111111111111111".parse().unwrap(),
            }.to_account_metas(None),
        )
    ], &[&payer2]);

    // --- Add liquidity so the pool has reserves ---
    let user_token_a = Keypair::new();
    let user_token_b = Keypair::new();
    let user_lp = Keypair::new();

    send_tx(&mut f.svm, &payer2, vec![
        system_instruction::create_account(&payer2.pubkey(), &user_token_a.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &user_token_a.pubkey(), &token_a_mint2.pubkey(), &payer2.pubkey()).unwrap(),
        system_instruction::create_account(&payer2.pubkey(), &user_token_b.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &user_token_b.pubkey(), &token_b_mint2.pubkey(), &payer2.pubkey()).unwrap(),
        system_instruction::create_account(&payer2.pubkey(), &user_lp.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &user_lp.pubkey(), &lp_mint2, &payer2.pubkey()).unwrap(),
    ], &[&payer2, &user_token_a, &user_token_b, &user_lp]);

    let liquidity_a: u64 = 100_000_000; // 100 tokens
    let liquidity_b: u64 = 100_000_000;

    send_tx(&mut f.svm, &payer2, vec![
        anchor_spl::token::spl_token::instruction::mint_to(&anchor_spl::token::ID, &token_a_mint2.pubkey(), &user_token_a.pubkey(), &payer2.pubkey(), &[], liquidity_a).unwrap(),
        anchor_spl::token::spl_token::instruction::mint_to(&anchor_spl::token::ID, &token_b_mint2.pubkey(), &user_token_b.pubkey(), &payer2.pubkey(), &[], liquidity_b).unwrap(),
    ], &[&payer2]);

    send_tx(&mut f.svm, &payer2, vec![
        Instruction::new_with_bytes(
            program_id,
            &exia_amm::instruction::AddLiquidity { amount_a: liquidity_a, amount_b: liquidity_b }.data(),
            exia_amm::accounts::AddLiquidity {
                user: payer2.pubkey(),
                pool_state: pool2_pda,
                user_token_a: user_token_a.pubkey(),
                user_token_b: user_token_b.pubkey(),
                user_lp_token: user_lp.pubkey(),
                vault_a: vault_a2,
                vault_b: vault_b2,
                lp_mint: lp_mint2,
                token_program: anchor_spl::token::ID,
                system_program: system_program::ID,
            }.to_account_metas(None),
        )
    ], &[&payer2]);

    // --- Setup swapper ---
    let swapper_token_a = Keypair::new();
    let swapper_token_b = Keypair::new();

    send_tx(&mut f.svm, &payer2, vec![
        system_instruction::create_account(&payer2.pubkey(), &swapper_token_a.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &swapper_token_a.pubkey(), &token_a_mint2.pubkey(), &payer2.pubkey()).unwrap(),
        system_instruction::create_account(&payer2.pubkey(), &swapper_token_b.pubkey(), token_account_rent, token_account_space, &anchor_spl::token::ID),
        anchor_spl::token::spl_token::instruction::initialize_account(&anchor_spl::token::ID, &swapper_token_b.pubkey(), &token_b_mint2.pubkey(), &payer2.pubkey()).unwrap(),
    ], &[&payer2, &swapper_token_a, &swapper_token_b]);

    let swap_amount_in: u64 = 1_000_000; // 1 token in
    send_tx(&mut f.svm, &payer2, vec![
        anchor_spl::token::spl_token::instruction::mint_to(&anchor_spl::token::ID, &token_a_mint2.pubkey(), &swapper_token_a.pubkey(), &payer2.pubkey(), &[], swap_amount_in).unwrap(),
    ], &[&payer2]);

    // --- Execute swap A → B ---
    let swap_ix = Instruction::new_with_bytes(
        program_id,
        &exia_amm::instruction::Swap {
            amount_in: swap_amount_in,
            minimum_amount_out: 1,
            a_to_b: true,
        }.data(),
        exia_amm::accounts::Swap {
            user: payer2.pubkey(),
            pool_state: pool2_pda,
            user_token_in: swapper_token_a.pubkey(),
            user_token_out: swapper_token_b.pubkey(),
            vault_a: vault_a2,
            vault_b: vault_b2,
            treasury_token_in: treasury.pubkey(),
            token_program: anchor_spl::token::ID,
        }.to_account_metas(None),
    );

    let blockhash = f.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[swap_ix], Some(&payer2.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer2]).unwrap();
    let res = f.svm.send_transaction(tx);
    assert!(res.is_ok(), "swap failed: {:?}", res);

    // --- Verify ---
    // Swapper should have received Token B
    let swapper_b_data = f.svm.get_account(&swapper_token_b.pubkey()).unwrap().data;
    let swapper_b_state = anchor_spl::token::TokenAccount::try_deserialize(&mut swapper_b_data.as_slice()).unwrap();
    assert!(swapper_b_state.amount > 0, "Swapper should have received Token B");

    // Treasury should have received protocol fee
    let treasury_data = f.svm.get_account(&treasury.pubkey()).unwrap().data;
    let treasury_state = anchor_spl::token::TokenAccount::try_deserialize(&mut treasury_data.as_slice()).unwrap();
    assert!(treasury_state.amount > 0, "Treasury should have received protocol fee");

    // k_last should have grown (LP fee stayed in vault)
    let pool_data = f.svm.get_account(&pool2_pda).unwrap().data;
    let pool_state = exia_amm::state::PoolState::try_deserialize(&mut pool_data.as_slice()).unwrap();
    assert!(pool_state.k_last > liquidity_a as u128 * liquidity_b as u128, "k_last should grow after swap");

    println!("SUCCESS! Swap A→B passed.");
    println!("  Token B received: {}", swapper_b_state.amount);
    println!("  Protocol fee collected: {}", treasury_state.amount);
    println!("  k_last after swap: {}", pool_state.k_last);
}
