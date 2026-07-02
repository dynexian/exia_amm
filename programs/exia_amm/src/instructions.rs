use crate::state::PoolState;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

/// Context context validator for establishing a brand new liquidity pool instance.
#[derive(Accounts)]
pub struct InitializePool<'info> {
    /// The initialization transaction signer and fee payer.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The zero-initialized state tracking account for the pool.
    /// Allocates `PoolState::MAX_SPACE` bytes on-chain under a deterministic seed structure.
    #[account(
        init,
        payer = payer,
        space = PoolState::MAX_SPACE,
        seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    /// The cryptographic mint identifier for Asset A.
    pub token_a_mint: Account<'info, Mint>,
    /// The cryptographic mint identifier for Asset B.
    pub token_b_mint: Account<'info, Mint>,

    /// The target protocol revenue account for collecting transaction cuts in Asset A.
    #[account(constraint = treasury_token_a.mint == token_a_mint.key())]
    pub treasury_token_a: Account<'info, TokenAccount>,
    
    /// The target protocol revenue account for collecting transaction cuts in Asset B.
    #[account(constraint = treasury_token_b.mint == token_b_mint.key())]
    pub treasury_token_b: Account<'info, TokenAccount>,

    /// The isolated system token vault constructed to store internal Token A reserves. Owned by the pool PDA.
    #[account(
        init,
        payer = payer,
        token::mint = token_a_mint,
        token::authority = pool_state,
        seeds = [b"vault_a", pool_state.key().as_ref()],
        bump,
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    /// The isolated system token vault constructed to store internal Token B reserves. Owned by the pool PDA.
    #[account(
        init,
        payer = payer,
        token::mint = token_b_mint,
        token::authority = pool_state,
        seeds = [b"vault_b", pool_state.key().as_ref()],
        bump,
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    /// The LP token mint ledger controlled exclusively by the pool state PDA.
    #[account(
        init,
        payer = payer,
        mint::decimals = 9,
        mint::authority = pool_state,
        seeds = [b"lp_mint", pool_state.key().as_ref()],
        bump,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// Core system program reference for PDA tracking.
    pub system_program: Program<'info, System>,
    /// Core SPL Token program reference for execution routing.
    pub token_program: Program<'info, Token>,
    /// System rent schedule reference.
    pub rent: Sysvar<'info, Rent>,
}

/// Context validator for execution steps within liquidity expansion.
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    /// The liquidity provider depositing raw assets.
    #[account(mut)]
    pub user: Signer<'info>,

    /// The targeted AMM pool state account configuration.
    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    /// User outbound ledger tracking account for Token A deposits.
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    /// User outbound ledger tracking account for Token B deposits.
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    /// User inbound ledger tracking account for receiving generated LP shares.
    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,

    /// The sovereign internal vault holding pool Token A assets.
    #[account(mut, address = pool_state.token_a_vault)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    /// The sovereign internal vault holding pool Token B assets.
    #[account(mut, address = pool_state.token_b_vault)]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    /// The global pool LP mint ledger.
    #[account(mut, address = pool_state.lp_mint)]
    pub lp_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Context validator tracking constraints during mathematical asset swapping.
#[derive(Accounts)]
pub struct Swap<'info> {
    /// The trader executing a curve exchange.
    #[account(mut)]
    pub user: Signer<'info>,

    /// The targeted AMM pool state account configuration.
    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    /// Trader outbound source account for asset disposal.
    #[account(mut)]
    pub user_token_in: Account<'info, TokenAccount>,
    /// Trader inbound destination account for asset collection.
    #[account(mut)]
    pub user_token_out: Account<'info, TokenAccount>,

    /// The sovereign internal vault holding pool Token A assets.
    #[account(mut, address = pool_state.token_a_vault)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    /// The sovereign internal vault holding pool Token B assets.
    #[account(mut, address = pool_state.token_b_vault)]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    /// The directionally valid treasury token account designated to trap the current trade's protocol fee.
    #[account(mut)]
    pub treasury_token_in: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// Context validator checking constraints during LP asset reclamation.
#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    /// The liquidity provider redeeming their token shares.
    #[account(mut)]
    pub user: Signer<'info>,

    /// The targeted AMM pool state account configuration.
    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    /// User inbound target account for reclaimed Token A distribution.
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    /// User inbound target account for reclaimed Token B distribution.
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    /// User outbound source account holding the LP tokens to be burned.
    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,

    /// The sovereign internal vault holding pool Token A assets.
    #[account(mut, address = pool_state.token_a_vault)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    /// The sovereign internal vault holding pool Token B assets.
    #[account(mut, address = pool_state.token_b_vault)]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    /// The global pool LP mint ledger.
    #[account(mut, address = pool_state.lp_mint)]
    pub lp_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

/// Context validator enforcing cryptographic admin signature matches for fee modifications.
#[derive(Accounts)]
pub struct UpdateFees<'info> {
    /// Privileged administrative signing wallet.
    pub authority: Signer<'info>,

    /// The pool configuration account verifying authority matches via the `has_one` check.
    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump,
        has_one = authority @ crate::error::ErrorCode::Unauthorized,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,
}

/// Context validator enforcing cryptographic admin signature matches for emergency operational switches.
#[derive(Accounts)]
pub struct SetPaused<'info> {
    /// Privileged administrative signing wallet.
    pub authority: Signer<'info>,

    /// The pool configuration account verifying authority matches via the `has_one` check.
    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump,
        has_one = authority @ crate::error::ErrorCode::Unauthorized,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,
}

/// Context validator enabling safe administrative modification of target revenue accounts.
#[derive(Accounts)]
pub struct RotateTreasury<'info> {
    /// Privileged administrative signing wallet.
    pub authority: Signer<'info>,

    /// The pool configuration account verifying authority matches via the `has_one` check.
    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump,
        has_one = authority @ crate::error::ErrorCode::Unauthorized,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    /// The replacement on-chain token account layout intended to trap subsequent Token A fees.
    #[account(constraint = new_treasury_token_a.mint == pool_state.token_a_mint)]
    pub new_treasury_token_a: Account<'info, TokenAccount>,
    
    /// The replacement on-chain token account layout intended to trap subsequent Token B fees.
    #[account(constraint = new_treasury_token_b.mint == pool_state.token_b_mint)]
    pub new_treasury_token_b: Account<'info, TokenAccount>,
}

/// Context validator for executing the initial phase of administrative ownership transition.
#[derive(Accounts)]
pub struct ProposeAuthority<'info> {
    /// Privileged administrative signing wallet.
    pub authority: Signer<'info>,

    /// The pool configuration account verifying authority matches via the `has_one` check.
    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump,
        has_one = authority @ crate::error::ErrorCode::Unauthorized,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,
}

/// Context validator for completing administrative ownership transition.
#[derive(Accounts)]
pub struct AcceptAuthority<'info> {
    /// The nominated administrative signing wallet accepting administrative command.
    pub new_authority: Signer<'info>,

    /// The pool configuration account receiving status mutation.
    #[account(
        mut,
        seeds = [b"pool", pool_state.token_a_mint.as_ref(), pool_state.token_b_mint.as_ref()],
        bump = pool_state.pool_bump,
    )]
    pub pool_state: Box<Account<'info, PoolState>>,
}