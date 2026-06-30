use anchor_lang::prelude::*;

#[account]
pub struct PoolState {
    // --- Architectural & PDA Safety ---
    pub token_a_mint: Pubkey,   // 32 bytes: The address of Token A's Mint
    pub token_b_mint: Pubkey,   // 32 bytes: The address of Token B's Mint
    pub token_a_vault: Pubkey,  // 32 bytes: The PDA Token Account holding Token A
    pub token_b_vault: Pubkey,  // 32 bytes: The PDA Token Account holding Token B
    pub lp_mint: Pubkey,        // 32 bytes: The LP Mint Account address

    // --- Canonical Bumps ---
    pub pool_bump: u8,          // 1 byte: Bump for this state account
    pub authority_bump: u8,     // 1 byte: Bump for the vault authority PDA

    // --- Fees Configuration ---
    pub lp_fee_bps: u16,        // 2 bytes: Fee accruing to LPs in Basis Points (e.g., 25 = 0.25%)
    pub protocol_fee_bps: u16,  // 2 bytes: Fee accruing to treasury in Basis Points (e.g., 5 = 0.05%)
    pub treasury_wallet: Pubkey,// 32 bytes: Wallet address where protocol fees are collected

    // --- Math & Invariant Tracking ---
    pub k_last: u128,           // 16 bytes: Product of reserves (x * y) right after the last liquidity event

    // --- TWAP Price Oracle ---
    pub price_a_cumulative_last: u128, // 16 bytes: Time-weighted cumulative price of Token A
    pub price_b_cumulative_last: u128, // 16 bytes: Time-weighted cumulative price of Token B
    pub block_timestamp_last: u64,     // 8 bytes: Timestamp of the last block a trade occurred
}

impl PoolState {
    // 8 (Discriminator) + 32*6 (Pubkeys) + 1*2 (u8s) + 2*2 (u16s) + 16*3 (u128s) + 8 (u64)
    pub const MAX_SPACE: usize = 262;
}
