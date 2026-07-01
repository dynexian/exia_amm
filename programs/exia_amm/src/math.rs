use anchor_lang::prelude::*;
use crate::error::ErrorCode;

pub const MINIMUM_LIQUIDITY: u64 = 1000;
pub const BPS_DENOMINATOR: u128 = 10_000;

pub fn calculate_lp_shares(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    total_lp_supply: u64,
) -> Result<u64> {
    if total_lp_supply == 0 {
        let product = (amount_a as u128)
            .checked_mul(amount_b as u128)
            .ok_or(ErrorCode::MathOverflow)?;
        let initial_shares = (product as f64).sqrt() as u64;
        if initial_shares < MINIMUM_LIQUIDITY {
            return Err(ErrorCode::InsufficientLiquidity.into());
        }
        Ok(initial_shares.checked_sub(MINIMUM_LIQUIDITY).unwrap())
    } else {
        let share_a = (amount_a as u128)
            .checked_mul(total_lp_supply as u128).unwrap()
            .checked_div(reserve_a as u128).unwrap();
        let share_b = (amount_b as u128)
            .checked_mul(total_lp_supply as u128).unwrap()
            .checked_div(reserve_b as u128).unwrap();
        Ok(std::cmp::min(share_a, share_b) as u64)
    }
}

/// Returns (amount_out, protocol_fee, lp_fee)
pub fn calculate_swap_output(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    lp_fee_bps: u16,
    protocol_fee_bps: u16,
) -> Result<(u64, u64, u64)> {
    if reserve_in == 0 || reserve_out == 0 {
        return Err(ErrorCode::InsufficientLiquidity.into());
    }

    let amount_in    = amount_in as u128;
    let reserve_in   = reserve_in as u128;
    let reserve_out  = reserve_out as u128;

    let protocol_fee = amount_in
        .checked_mul(protocol_fee_bps as u128).ok_or(ErrorCode::MathOverflow)?
        .checked_div(BPS_DENOMINATOR).ok_or(ErrorCode::MathOverflow)?;

    let lp_fee = amount_in
        .checked_mul(lp_fee_bps as u128).ok_or(ErrorCode::MathOverflow)?
        .checked_div(BPS_DENOMINATOR).ok_or(ErrorCode::MathOverflow)?;

    let tradeable = amount_in
        .checked_sub(protocol_fee).ok_or(ErrorCode::MathOverflow)?
        .checked_sub(lp_fee).ok_or(ErrorCode::MathOverflow)?;

    if tradeable == 0 {
        return Err(ErrorCode::InsufficientLiquidity.into());
    }

    let numerator   = reserve_out.checked_mul(tradeable).ok_or(ErrorCode::MathOverflow)?;
    let denominator = reserve_in.checked_add(tradeable).ok_or(ErrorCode::MathOverflow)?;
    let amount_out  = numerator.checked_div(denominator).ok_or(ErrorCode::MathOverflow)?;

    Ok((amount_out as u64, protocol_fee as u64, lp_fee as u64))
}

/// Returns (amount_a_out, amount_b_out)
pub fn calculate_remove_liquidity(
    lp_amount: u64,
    total_lp_supply: u64,
    reserve_a: u64,
    reserve_b: u64,
) -> Result<(u64, u64)> {
    if total_lp_supply == 0 {
        return Err(ErrorCode::NoLiquidity.into());
    }
    let amount_a = (lp_amount as u128)
        .checked_mul(reserve_a as u128).ok_or(ErrorCode::MathOverflow)?
        .checked_div(total_lp_supply as u128).ok_or(ErrorCode::MathOverflow)? as u64;
    let amount_b = (lp_amount as u128)
        .checked_mul(reserve_b as u128).ok_or(ErrorCode::MathOverflow)?
        .checked_div(total_lp_supply as u128).ok_or(ErrorCode::MathOverflow)? as u64;
    Ok((amount_a, amount_b))
}

/// Update cumulative prices using Q32.32 fixed-point arithmetic.
/// Returns (new_price_a_cumulative, new_price_b_cumulative, new_timestamp)
pub fn update_twap(
    price_a_cumulative_last: u128,
    price_b_cumulative_last: u128,
    block_timestamp_last: u64,
    reserve_a: u64,
    reserve_b: u64,
    current_timestamp: u64,
) -> Result<(u128, u128, u64)> {
    if reserve_a == 0 || reserve_b == 0 {
        return Err(ErrorCode::NoLiquidity.into());
    }

    let elapsed = current_timestamp.saturating_sub(block_timestamp_last);

    // If no time has passed (same block), don't update to avoid division issues
    if elapsed == 0 {
        return Ok((price_a_cumulative_last, price_b_cumulative_last, block_timestamp_last));
    }

    // Q32.32 fixed-point: shift numerator left by 32 bits before dividing
    // price_a = reserve_b / reserve_a (how much B per unit of A)
    let price_a_q32 = ((reserve_b as u128) << 32)
        .checked_div(reserve_a as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    // price_b = reserve_a / reserve_b (how much A per unit of B)
    let price_b_q32 = ((reserve_a as u128) << 32)
        .checked_div(reserve_b as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    let new_price_a_cumulative = price_a_cumulative_last
        .wrapping_add(price_a_q32.wrapping_mul(elapsed as u128));

    let new_price_b_cumulative = price_b_cumulative_last
        .wrapping_add(price_b_q32.wrapping_mul(elapsed as u128));

    Ok((new_price_a_cumulative, new_price_b_cumulative, current_timestamp))
}
