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

    /// CHECK: This is a Mint account. We only need its public key to derive the PDA seeds.
    /// The program logic does not read or write data from this account, so no type check is required.
    pub token_a_mint: UncheckedAccount<'info>,

    /// CHECK: This is a Mint account. We only need its public key to derive the PDA seeds.
    /// The program logic does not read or write data from this account, so no type check is required.
    pub token_b_mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
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