use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Transfer};

pub mod constants;
pub mod error;
pub mod instructions;
pub mod math;
pub mod state;

use error::ErrorCode;
use instructions::*;

declare_id!("2Hy7ouFwJLkG7cpAoSR4hGaFk3zPH2gAYLKjTMdGsqQs");

fn current_timestamp() -> Result<u64> {
    Ok(Clock::get()?.unix_timestamp.max(0) as u64)
}

fn sync_twap(
    pool_state: &mut state::PoolState,
    reserve_a: u64,
    reserve_b: u64,
    current_ts: u64,
) -> Result<()> {
    if reserve_a == 0 || reserve_b == 0 {
        pool_state.block_timestamp_last = current_ts;
        return Ok(());
    }

    let (new_price_a_cum, new_price_b_cum, new_ts) = math::update_twap(
        pool_state.price_a_cumulative_last,
        pool_state.price_b_cumulative_last,
        pool_state.block_timestamp_last,
        reserve_a,
        reserve_b,
        current_ts,
    )?;
    pool_state.price_a_cumulative_last = new_price_a_cum;
    pool_state.price_b_cumulative_last = new_price_b_cum;
    pool_state.block_timestamp_last = new_ts;
    Ok(())
}

#[program]
pub mod exia_amm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        lp_fee_bps: u16,
        protocol_fee_bps: u16,
        authority: Pubkey,
    ) -> Result<()> {
        require!(lp_fee_bps <= 500, ErrorCode::FeeTooHigh);
        require!(protocol_fee_bps <= 500, ErrorCode::FeeTooHigh);

        let pool_state = &mut ctx.accounts.pool_state;

        pool_state.token_a_mint = ctx.accounts.token_a_mint.key();
        pool_state.token_b_mint = ctx.accounts.token_b_mint.key();
        pool_state.token_a_vault = ctx.accounts.vault_a.key();
        pool_state.token_b_vault = ctx.accounts.vault_b.key();
        pool_state.lp_mint = ctx.accounts.lp_mint.key();
        pool_state.treasury_token_a = ctx.accounts.treasury_token_a.key();
        pool_state.treasury_token_b = ctx.accounts.treasury_token_b.key();
        pool_state.authority = authority;
        pool_state.pending_authority = Pubkey::default();
        pool_state.is_paused = false;

        pool_state.pool_bump = ctx.bumps.pool_state;
        pool_state.authority_bump = 0;

        pool_state.lp_fee_bps = lp_fee_bps;
        pool_state.protocol_fee_bps = protocol_fee_bps;

        pool_state.k_last = 0;
        pool_state.price_a_cumulative_last = 0;
        pool_state.price_b_cumulative_last = 0;
        pool_state.block_timestamp_last = current_timestamp()?;

        Ok(())
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        require!(
            !ctx.accounts.pool_state.is_paused,
            crate::error::ErrorCode::PoolPaused
        );
        let reserve_a_before = ctx.accounts.vault_a.amount;
        let reserve_b_before = ctx.accounts.vault_b.amount;
        let total_lp_supply = ctx.accounts.lp_mint.supply;
        let current_ts = current_timestamp()?;
        sync_twap(
            &mut ctx.accounts.pool_state,
            reserve_a_before,
            reserve_b_before,
            current_ts,
        )?;

        let shares_to_mint = math::calculate_lp_shares(
            amount_a,
            amount_b,
            reserve_a_before,
            reserve_b_before,
            total_lp_supply,
        )?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_a,
        )?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_b,
        )?;

        ctx.accounts.vault_a.reload()?;
        ctx.accounts.vault_b.reload()?;

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.user_lp_token.to_account_info(),
                    authority: ctx.accounts.pool_state.to_account_info(),
                },
                &[&[
                    b"pool",
                    ctx.accounts.pool_state.token_a_mint.as_ref(),
                    ctx.accounts.pool_state.token_b_mint.as_ref(),
                    &[ctx.accounts.pool_state.pool_bump],
                ]],
            ),
            shares_to_mint,
        )?;

        ctx.accounts.pool_state.k_last = (ctx.accounts.vault_a.amount as u128)
            .checked_mul(ctx.accounts.vault_b.amount as u128)
            .ok_or(crate::error::ErrorCode::MathOverflow)?;

        Ok(())
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        minimum_amount_out: u64,
        a_to_b: bool,
    ) -> Result<()> {
        require!(
            !ctx.accounts.pool_state.is_paused,
            crate::error::ErrorCode::PoolPaused
        );
        let reserve_a_before = ctx.accounts.vault_a.amount;
        let reserve_b_before = ctx.accounts.vault_b.amount;

        // Load reserves based on direction
        let (reserve_in, reserve_out) = if a_to_b {
            (reserve_a_before, reserve_b_before)
        } else {
            (reserve_b_before, reserve_a_before)
        };

        let (expected_user_in_mint, expected_user_out_mint, expected_treasury) = if a_to_b {
            (
                ctx.accounts.pool_state.token_a_mint,
                ctx.accounts.pool_state.token_b_mint,
                ctx.accounts.pool_state.treasury_token_a,
            )
        } else {
            (
                ctx.accounts.pool_state.token_b_mint,
                ctx.accounts.pool_state.token_a_mint,
                ctx.accounts.pool_state.treasury_token_b,
            )
        };

        require_keys_eq!(
            ctx.accounts.user_token_in.mint,
            expected_user_in_mint,
            ErrorCode::InvalidTokenAccount
        );
        require_keys_eq!(
            ctx.accounts.user_token_out.mint,
            expected_user_out_mint,
            ErrorCode::InvalidTokenAccount
        );
        require_keys_eq!(
            ctx.accounts.treasury_token_in.key(),
            expected_treasury,
            ErrorCode::InvalidTreasury
        );
        require_keys_eq!(
            ctx.accounts.treasury_token_in.mint,
            expected_user_in_mint,
            ErrorCode::InvalidTreasury
        );

        // Step 1: Calculate output and fee amounts
        let (amount_out, protocol_fee, _lp_fee) = math::calculate_swap_output(
            amount_in,
            reserve_in,
            reserve_out,
            ctx.accounts.pool_state.lp_fee_bps,
            ctx.accounts.pool_state.protocol_fee_bps,
        )?;

        // Step 2: Slippage check
        require!(
            amount_out >= minimum_amount_out,
            ErrorCode::SlippageExceeded
        );

        let current_ts = current_timestamp()?;
        sync_twap(
            &mut ctx.accounts.pool_state,
            reserve_a_before,
            reserve_b_before,
            current_ts,
        )?;

        let pool_seeds = &[
            b"pool",
            ctx.accounts.pool_state.token_a_mint.as_ref(),
            ctx.accounts.pool_state.token_b_mint.as_ref(),
            &[ctx.accounts.pool_state.pool_bump],
        ];

        // Step 3a: Transfer full amount_in from user → vault_in
        let (vault_in, vault_out) = if a_to_b {
            (
                ctx.accounts.vault_a.to_account_info(),
                ctx.accounts.vault_b.to_account_info(),
            )
        } else {
            (
                ctx.accounts.vault_b.to_account_info(),
                ctx.accounts.vault_a.to_account_info(),
            )
        };

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.user_token_in.to_account_info(),
                    to: vault_in,
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_in,
        )?;

        // Step 3b: Transfer protocol_fee from vault_in → treasury
        // (pool_state PDA signs since it owns the vault)
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: if a_to_b {
                        ctx.accounts.vault_a.to_account_info()
                    } else {
                        ctx.accounts.vault_b.to_account_info()
                    },
                    to: ctx.accounts.treasury_token_in.to_account_info(),
                    authority: ctx.accounts.pool_state.to_account_info(),
                },
                &[pool_seeds],
            ),
            protocol_fee,
        )?;

        // Step 3c: Transfer amount_out from vault_out → user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: vault_out,
                    to: ctx.accounts.user_token_out.to_account_info(),
                    authority: ctx.accounts.pool_state.to_account_info(),
                },
                &[pool_seeds],
            ),
            amount_out,
        )?;

        // Step 4: Reload reserves after the swap
        ctx.accounts.vault_a.reload()?;
        ctx.accounts.vault_b.reload()?;

        // Step 5: Update invariant snapshot
        ctx.accounts.pool_state.k_last = (ctx.accounts.vault_a.amount as u128)
            .checked_mul(ctx.accounts.vault_b.amount as u128)
            .ok_or(crate::error::ErrorCode::MathOverflow)?;

        Ok(())
    }
    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lp_amount: u64) -> Result<()> {
        let total_supply = ctx.accounts.lp_mint.supply;
        let reserve_a = ctx.accounts.vault_a.amount;
        let reserve_b = ctx.accounts.vault_b.amount;
        let current_ts = current_timestamp()?;
        sync_twap(
            &mut ctx.accounts.pool_state,
            reserve_a,
            reserve_b,
            current_ts,
        )?;

        let (amount_a_out, amount_b_out) =
            math::calculate_remove_liquidity(lp_amount, total_supply, reserve_a, reserve_b)?;

        let pool_seeds = &[
            b"pool",
            ctx.accounts.pool_state.token_a_mint.as_ref(),
            ctx.accounts.pool_state.token_b_mint.as_ref(),
            &[ctx.accounts.pool_state.pool_bump],
        ];

        // Burn LP tokens first
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                token::Burn {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    from: ctx.accounts.user_lp_token.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            lp_amount,
        )?;

        // Return Token A to user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.vault_a.to_account_info(),
                    to: ctx.accounts.user_token_a.to_account_info(),
                    authority: ctx.accounts.pool_state.to_account_info(),
                },
                &[pool_seeds],
            ),
            amount_a_out,
        )?;

        // Return Token B to user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.vault_b.to_account_info(),
                    to: ctx.accounts.user_token_b.to_account_info(),
                    authority: ctx.accounts.pool_state.to_account_info(),
                },
                &[pool_seeds],
            ),
            amount_b_out,
        )?;

        // Update invariant
        ctx.accounts.vault_a.reload()?;
        ctx.accounts.vault_b.reload()?;
        ctx.accounts.pool_state.k_last = (ctx.accounts.vault_a.amount as u128)
            .checked_mul(ctx.accounts.vault_b.amount as u128)
            .ok_or(crate::error::ErrorCode::MathOverflow)?;

        Ok(())
    }

    pub fn update_fees(
        ctx: Context<UpdateFees>,
        new_lp_fee_bps: u16,
        new_protocol_fee_bps: u16,
    ) -> Result<()> {
        require!(new_lp_fee_bps <= 500, crate::error::ErrorCode::FeeTooHigh);
        require!(
            new_protocol_fee_bps <= 500,
            crate::error::ErrorCode::FeeTooHigh
        );
        let pool_state = &mut ctx.accounts.pool_state;
        pool_state.lp_fee_bps = new_lp_fee_bps;
        pool_state.protocol_fee_bps = new_protocol_fee_bps;
        Ok(())
    }

    pub fn set_paused(ctx: Context<SetPaused>, paused: bool) -> Result<()> {
        ctx.accounts.pool_state.is_paused = paused;
        Ok(())
    }

    pub fn rotate_treasury(ctx: Context<RotateTreasury>) -> Result<()> {
        ctx.accounts.pool_state.treasury_token_a = ctx.accounts.new_treasury_token_a.key();
        ctx.accounts.pool_state.treasury_token_b = ctx.accounts.new_treasury_token_b.key();
        Ok(())
    }

    pub fn propose_authority(ctx: Context<ProposeAuthority>, new_authority: Pubkey) -> Result<()> {
        ctx.accounts.pool_state.pending_authority = new_authority;
        Ok(())
    }

    pub fn accept_authority(ctx: Context<AcceptAuthority>) -> Result<()> {
        let pool_state = &mut ctx.accounts.pool_state;
        require!(
            pool_state.pending_authority == ctx.accounts.new_authority.key(),
            crate::error::ErrorCode::Unauthorized
        );
        pool_state.authority = ctx.accounts.new_authority.key();
        pool_state.pending_authority = Pubkey::default();
        Ok(())
    }
}
