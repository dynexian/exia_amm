use anchor_lang::prelude::*;
use crate::state::PoolState;
use anchor_spl::token::{Token, TokenAccount, Mint};
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = PoolState::MAX_SPACE,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    // Upgraded to actual Mint types so Anchor can validate them
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,

    // --- PROTOCOL SOVEREIGNTY: Auto-creating the Vaults and LP Mint ---

    #[account(
        init,
        payer = payer,
        token::mint = token_a_mint,
        token::authority = pool_state, // The PDA owns this vault
        seeds = [b"vault_a", pool_state.key().as_ref()],
        bump,
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        token::mint = token_b_mint,
        token::authority = pool_state,
        seeds = [b"vault_b", pool_state.key().as_ref()],
        bump,
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 9,
        mint::authority = pool_state, // The PDA controls the LP printing press
        seeds = [b"lp_mint", pool_state.key().as_ref()],
        bump,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    // Necessary programs for creating token accounts
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>, // Wrapped in Box

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,

    #[account(mut, address = pool_state.token_a_vault)]
    pub vault_a: Box<Account<'info, TokenAccount>>, // Wrapped in Box
    #[account(mut, address = pool_state.token_b_vault)]
    pub vault_b: Box<Account<'info, TokenAccount>>, // Wrapped in Box

    #[account(mut, address = pool_state.lp_mint)]
    pub lp_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    // User's source and destination token accounts (direction determined by a_to_b flag)
    #[account(mut)]
    pub user_token_in: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_out: Account<'info, TokenAccount>,

    // Vaults — both needed regardless of direction
    #[account(mut, address = pool_state.token_a_vault)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = pool_state.token_b_vault)]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    /// CHECK: This is the treasury wallet stored in pool_state.
    /// We verify its address via the constraint below.
    #[account(
        mut,
        address = pool_state.treasury_wallet
    )]
    pub treasury_token_in: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,

    #[account(mut, address = pool_state.token_a_vault)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = pool_state.token_b_vault)]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = pool_state.lp_mint)]
    pub lp_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}
