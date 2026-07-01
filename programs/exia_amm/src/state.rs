use anchor_lang::prelude::*;

#[account]
pub struct PoolState {
    // --- Architectural & PDA Safety ---
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub lp_mint: Pubkey,

    // --- Canonical Bumps ---
    pub pool_bump: u8,
    pub authority_bump: u8,

    // --- Fees Configuration ---
    pub lp_fee_bps: u16,
    pub protocol_fee_bps: u16,
    pub treasury_wallet: Pubkey,

    // --- Admin ---
    pub authority: Pubkey,
    pub pending_authority: Pubkey,  // zero pubkey = no pending transfer
    pub is_paused: bool,

    // --- Math & Invariant Tracking ---
    pub k_last: u128,

    // --- TWAP Price Oracle ---
    pub price_a_cumulative_last: u128,
    pub price_b_cumulative_last: u128,
    pub block_timestamp_last: u64,
}

impl PoolState {
    // 8 + 32*7 + 1*2 + 2*2 + 1 + 16*3 + 8
    // 8 + 32*8 + 1*2 + 2*2 + 1 + 16*3 + 8
    pub const MAX_SPACE: usize = 327;
}
