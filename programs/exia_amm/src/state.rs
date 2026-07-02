use anchor_lang::prelude::*;

/// Core state account for an Exia AMM liquidity pool.
/// Operates on a Constant Product Market Maker (CPMM) curve.
#[account]
pub struct PoolState {
    // --- Architectural & PDA Safety ---
    
    /// The SPL Token Mint for the pool's first asset.
    pub token_a_mint: Pubkey,
    /// The SPL Token Mint for the pool's second asset.
    pub token_b_mint: Pubkey,
    /// The PDA Token Account holding the pool's Token A reserves.
    pub token_a_vault: Pubkey,
    /// The PDA Token Account holding the pool's Token B reserves.
    pub token_b_vault: Pubkey,
    /// The PDA Mint Account for the pool's Liquidity Provider (LP) tokens.
    pub lp_mint: Pubkey,

    // --- Canonical Bumps ---
    
    /// The bump seed used to derive this PoolState PDA.
    pub pool_bump: u8,
    /// The bump seed for the vault authority (currently unused, reserved for future layout stability).
    pub authority_bump: u8,

    // --- Fees Configuration ---
    
    /// Fee accruing to Liquidity Providers, measured in Basis Points (1 bps = 0.01%).
    pub lp_fee_bps: u16,
    /// Fee accruing to the protocol treasury, measured in Basis Points.
    pub protocol_fee_bps: u16,
    /// The external Token Account designated to receive Token A protocol fees.
    pub treasury_token_a: Pubkey,
    /// The external Token Account designated to receive Token B protocol fees.
    pub treasury_token_b: Pubkey,

    // --- Admin & Access Control ---
    
    /// The privileged wallet capable of pausing the pool or modifying fees/treasuries.
    pub authority: Pubkey,
    /// The proposed next authority. Requires a two-step acceptance to prevent accidental lockouts.
    pub pending_authority: Pubkey,
    /// Emergency kill-switch. If true, swaps and liquidity additions are blocked.
    pub is_paused: bool,

    // --- Math & Invariant Tracking ---
    
    /// The constant product invariant snapshot ($x \cdot y = k$) captured after the last liquidity event.
    pub k_last: u128,

    // --- TWAP Price Oracle ---
    
    /// Time-weighted cumulative price of Token A (Q32.32 fixed-point representation).
    pub price_a_cumulative_last: u128,
    /// Time-weighted cumulative price of Token B (Q32.32 fixed-point representation).
    pub price_b_cumulative_last: u128,
    /// The Unix timestamp of the block when the TWAP oracle was last synced.
    pub block_timestamp_last: u64,
}

impl PoolState {
    /// Exact byte size required for rent exemption allocation.
    /// 8 (discriminator) + 9*32 (pubkeys) + 2*1 (u8) + 2*2 (u16) + 1 (bool) + 3*16 (u128) + 8 (u64) = 359 bytes
    pub const MAX_SPACE: usize = 359;
}