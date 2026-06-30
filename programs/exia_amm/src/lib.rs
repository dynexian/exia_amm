use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, MintTo};

pub mod state;
pub mod instructions;
pub mod math;
pub mod constants;
pub mod error;

use instructions::*;

declare_id!("2Hy7ouFwJLkG7cpAoSR4hGaFk3zPH2gAYLKjTMdGsqQs");
#[program]
pub mod exia_amm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        lp_fee_bps: u16,
        protocol_fee_bps: u16
    ) -> Result<()> {
        let pool_state = &mut ctx.accounts.pool_state;

        pool_state.token_a_mint = ctx.accounts.token_a_mint.key();
        pool_state.token_b_mint = ctx.accounts.token_b_mint.key();

        pool_state.token_a_vault = ctx.accounts.vault_a.key();
        pool_state.token_b_vault = ctx.accounts.vault_b.key();
        pool_state.lp_mint = ctx.accounts.lp_mint.key();

        pool_state.pool_bump = ctx.bumps.pool_state;
        // The pool_state PDA is its own authority, so we save its bump for signing later
        pool_state.authority_bump = ctx.bumps.pool_state;

        pool_state.lp_fee_bps = lp_fee_bps;
        pool_state.protocol_fee_bps = protocol_fee_bps;

        pool_state.k_last = 0;
        pool_state.price_a_cumulative_last = 0;
        pool_state.price_b_cumulative_last = 0;
        pool_state.block_timestamp_last = 0;

        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {

        // --- 1. TRANSFER TOKEN A ---
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

        // --- 2. TRANSFER TOKEN B ---
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

        // --- 3. CALCULATE LP SHARES ---
        ctx.accounts.vault_a.reload()?;
        ctx.accounts.vault_b.reload()?;

        let shares_to_mint = math::calculate_lp_shares(
            amount_a,
            amount_b,
            ctx.accounts.vault_a.amount,
            ctx.accounts.vault_b.amount,
            ctx.accounts.lp_mint.supply,
        )?;

        // --- 4. MINT LP TOKENS TO USER ---
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

        // --- 5. UPDATE INVARIANT ---
        ctx.accounts.pool_state.k_last = (ctx.accounts.vault_a.amount as u128)
            .checked_mul(ctx.accounts.vault_b.amount as u128)
            .ok_or(math::ErrorCode::MathOverflow)?;

        Ok(())
    }
}