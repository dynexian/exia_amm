use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, MintTo};

pub mod state;
pub mod instructions;
pub mod math;
pub mod constants;
pub mod error;

use instructions::*;
use error::ErrorCode;

declare_id!("2Hy7ouFwJLkG7cpAoSR4hGaFk3zPH2gAYLKjTMdGsqQs");

#[program]
pub mod exia_amm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        lp_fee_bps: u16,
        protocol_fee_bps: u16,
        treasury_wallet: Pubkey,
    ) -> Result<()> {
        let pool_state = &mut ctx.accounts.pool_state;

        pool_state.token_a_mint    = ctx.accounts.token_a_mint.key();
        pool_state.token_b_mint    = ctx.accounts.token_b_mint.key();
        pool_state.token_a_vault   = ctx.accounts.vault_a.key();
        pool_state.token_b_vault   = ctx.accounts.vault_b.key();
        pool_state.lp_mint         = ctx.accounts.lp_mint.key();
        pool_state.treasury_wallet = treasury_wallet;

        pool_state.pool_bump       = ctx.bumps.pool_state;
        pool_state.authority_bump  = 0;

        pool_state.lp_fee_bps      = lp_fee_bps;
        pool_state.protocol_fee_bps = protocol_fee_bps;

        pool_state.k_last                   = 0;
        pool_state.price_a_cumulative_last  = 0;
        pool_state.price_b_cumulative_last  = 0;
        pool_state.block_timestamp_last     = 0;

        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from:      ctx.accounts.user_token_a.to_account_info(),
                    to:        ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_a,
        )?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from:      ctx.accounts.user_token_b.to_account_info(),
                    to:        ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_b,
        )?;

        ctx.accounts.vault_a.reload()?;
        ctx.accounts.vault_b.reload()?;

        let shares_to_mint = math::calculate_lp_shares(
            amount_a,
            amount_b,
            ctx.accounts.vault_a.amount,
            ctx.accounts.vault_b.amount,
            ctx.accounts.lp_mint.supply,
        )?;

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                MintTo {
                    mint:      ctx.accounts.lp_mint.to_account_info(),
                    to:        ctx.accounts.user_lp_token.to_account_info(),
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
        let pool_state = &ctx.accounts.pool_state;

        // Load reserves based on direction
        let (reserve_in, reserve_out) = if a_to_b {
            (ctx.accounts.vault_a.amount, ctx.accounts.vault_b.amount)
        } else {
            (ctx.accounts.vault_b.amount, ctx.accounts.vault_a.amount)
        };

        // Step 1: Calculate output and fee amounts
        let (amount_out, protocol_fee, _lp_fee) = math::calculate_swap_output(
            amount_in,
            reserve_in,
            reserve_out,
            pool_state.lp_fee_bps,
            pool_state.protocol_fee_bps,
        )?;

        // Step 2: Slippage check
        require!(amount_out >= minimum_amount_out, ErrorCode::SlippageExceeded);

        let pool_seeds = &[
            b"pool",
            pool_state.token_a_mint.as_ref(),
            pool_state.token_b_mint.as_ref(),
            &[pool_state.pool_bump],
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
                    from:      ctx.accounts.user_token_in.to_account_info(),
                    to:        vault_in,
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
                    from:      if a_to_b {
                        ctx.accounts.vault_a.to_account_info()
                    } else {
                        ctx.accounts.vault_b.to_account_info()
                    },
                    to:        ctx.accounts.treasury_token_in.to_account_info(),
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
                    from:      vault_out,
                    to:        ctx.accounts.user_token_out.to_account_info(),
                    authority: ctx.accounts.pool_state.to_account_info(),
                },
                &[pool_seeds],
            ),
            amount_out,
        )?;


        // Step 4: Update TWAP accumulators
        ctx.accounts.vault_a.reload()?;
        ctx.accounts.vault_b.reload()?;

        let current_ts = Clock::get()?.unix_timestamp as u64;
        let (new_price_a_cum, new_price_b_cum, new_ts) = math::update_twap(
            ctx.accounts.pool_state.price_a_cumulative_last,
            ctx.accounts.pool_state.price_b_cumulative_last,
            ctx.accounts.pool_state.block_timestamp_last,
            ctx.accounts.vault_a.amount,
            ctx.accounts.vault_b.amount,
            current_ts,
        )?;
        ctx.accounts.pool_state.price_a_cumulative_last = new_price_a_cum;
        ctx.accounts.pool_state.price_b_cumulative_last = new_price_b_cum;
        ctx.accounts.pool_state.block_timestamp_last = new_ts;

        // Step 5: Update invariant snapshot
        ctx.accounts.pool_state.k_last = (ctx.accounts.vault_a.amount as u128)
            .checked_mul(ctx.accounts.vault_b.amount as u128)
            .ok_or(crate::error::ErrorCode::MathOverflow)?;

        Ok(())
    }
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
    ) -> Result<()> {
        let total_supply = ctx.accounts.lp_mint.supply;
        let reserve_a = ctx.accounts.vault_a.amount;
        let reserve_b = ctx.accounts.vault_b.amount;

        let (amount_a_out, amount_b_out) = math::calculate_remove_liquidity(
            lp_amount,
            total_supply,
            reserve_a,
            reserve_b,
        )?;

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

}
// append-sentinel — remove this line, paste block inside #[program] mod manually
