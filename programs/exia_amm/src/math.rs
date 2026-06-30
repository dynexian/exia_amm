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
