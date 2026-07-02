use crate::error::ErrorCode;
use anchor_lang::prelude::*;

/// The global basis point denominator where 10,000 equals 100%.
pub const BPS_DENOMINATOR: u128 = 10_000;

/// Computes the floor integer square root of a given $u128$ number using the Babylonian method.
/// 
/// # Security
/// - Operates entirely within the integer domain to guarantee zero precision loss from float rendering.
fn integer_sqrt(value: u128) -> u128 {
    if value < 2 {
        return value;
    }

    let mut x0 = value / 2;
    let mut x1 = (x0 + value / x0) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x0 + value / x0) / 2;
    }
    x0
}

/// Calculates the exact number of LP shares to mint for a given liquidity deposit.
/// 
/// # Mathematical Model
/// - Initial Deposit: $\text{shares} = \lfloor\sqrt{\Delta x \cdot \Delta y}\rfloor$
/// - Subsequent Deposits: $\text{shares} = \min\left(\frac{\Delta x \cdot S}{x}, \frac{\Delta y \cdot S}{y}\right)$
/// 
/// # Security
/// - Enforces that deposits maintain the exact balance ratio of the existing reserves to prevent arbitrage exploitation.
pub fn calculate_lp_shares(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    total_lp_supply: u64,
) -> Result<u64> {
    if amount_a == 0 || amount_b == 0 {
        return Err(ErrorCode::InvalidAmount.into());
    }

    if total_lp_supply == 0 {
        let product = (amount_a as u128)
            .checked_mul(amount_b as u128)
            .ok_or(ErrorCode::MathOverflow)?;
        let initial_shares = integer_sqrt(product);
        if initial_shares == 0 {
            return Err(ErrorCode::InsufficientLiquidity.into());
        }
        require!(initial_shares <= u64::MAX as u128, ErrorCode::MathOverflow);
        Ok(initial_shares as u64)
    } else {
        if reserve_a == 0 || reserve_b == 0 {
            return Err(ErrorCode::NoLiquidity.into());
        }
        let share_a = (amount_a as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(reserve_a as u128)
            .ok_or(ErrorCode::MathOverflow)?;
        let share_b = (amount_b as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(reserve_b as u128)
            .ok_or(ErrorCode::MathOverflow)?;
        let shares = std::cmp::min(share_a, share_b);
        require!(shares > 0, ErrorCode::InsufficientLiquidity);
        require!(shares <= u64::MAX as u128, ErrorCode::MathOverflow);
        Ok(shares as u64)
    }
}

/// Computes the swap execution output and breaks down the exact fee tracking structures.
/// 
/// # Siphoning Architecture
/// - Slices the fees off the top of the input token before passing the remainder to the curve engine.
/// - Returns `(amount_out, protocol_fee, lp_fee)`.
pub fn calculate_swap_output(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    lp_fee_bps: u16,
    protocol_fee_bps: u16,
) -> Result<(u64, u64, u64)> {
    if amount_in == 0 {
        return Err(ErrorCode::InvalidAmount.into());
    }
    if reserve_in == 0 || reserve_out == 0 {
        return Err(ErrorCode::InsufficientLiquidity.into());
    }

    let amount_in = amount_in as u128;
    let reserve_in = reserve_in as u128;
    let reserve_out = reserve_out as u128;

    let protocol_fee = amount_in
        .checked_mul(protocol_fee_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(ErrorCode::MathOverflow)?;

    let lp_fee = amount_in
        .checked_mul(lp_fee_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(ErrorCode::MathOverflow)?;

    let tradeable = amount_in
        .checked_sub(protocol_fee)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_sub(lp_fee)
        .ok_or(ErrorCode::MathOverflow)?;

    if tradeable == 0 {
        return Err(ErrorCode::InsufficientLiquidity.into());
    }

    let numerator = reserve_out
        .checked_mul(tradeable)
        .ok_or(ErrorCode::MathOverflow)?;
    let denominator = reserve_in
        .checked_add(tradeable)
        .ok_or(ErrorCode::MathOverflow)?;
    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(ErrorCode::MathOverflow)?;
    require!(amount_out > 0, ErrorCode::InsufficientLiquidity);
    require!(amount_out <= u64::MAX as u128, ErrorCode::MathOverflow);

    Ok((amount_out as u64, protocol_fee as u64, lp_fee as u64))
}

/// Calculates the proportional pro-rata share of underling vault assets to release back to an LP.
pub fn calculate_remove_liquidity(
    lp_amount: u64,
    total_lp_supply: u64,
    reserve_a: u64,
    reserve_b: u64,
) -> Result<(u64, u64)> {
    if lp_amount == 0 {
        return Err(ErrorCode::InvalidAmount.into());
    }
    if total_lp_supply == 0 {
        return Err(ErrorCode::NoLiquidity.into());
    }
    if lp_amount > total_lp_supply {
        return Err(ErrorCode::InvalidAmount.into());
    }
    let amount_a = (lp_amount as u128)
        .checked_mul(reserve_a as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(ErrorCode::MathOverflow)? as u64;
    let amount_b = (lp_amount as u128)
        .checked_mul(reserve_b as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(total_lp_supply as u128)
        .ok_or(ErrorCode::MathOverflow)? as u64;
    Ok((amount_a, amount_b))
}

/// Accumulates prices for the Time-Weighted Average Price (TWAP) calculation using $Q32.32$ fixed-point precision.
/// 
/// # Arithmetic Framework
/// - Shifts the numerator left by 32 bits before dividing to embed fractional price values into an integer format.
/// - Accumulations utilize wrapping addition to bypass capacity constraints safely over infinite operating timelines.
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

    if elapsed == 0 {
        return Ok((
            price_a_cumulative_last,
            price_b_cumulative_last,
            block_timestamp_last,
        ));
    }

    let price_a_q32 = ((reserve_b as u128) << 32)
        .checked_div(reserve_a as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    let price_b_q32 = ((reserve_a as u128) << 32)
        .checked_div(reserve_b as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    let new_price_a_cumulative =
        price_a_cumulative_last.wrapping_add(price_a_q32.wrapping_mul(elapsed as u128));

    let new_price_b_cumulative =
        price_b_cumulative_last.wrapping_add(price_b_q32.wrapping_mul(elapsed as u128));

    Ok((
        new_price_a_cumulative,
        new_price_b_cumulative,
        current_timestamp,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_lp_shares_use_floor_sqrt() {
        let shares = calculate_lp_shares(10, 30, 0, 0, 0).unwrap();
        assert_eq!(shares, 17);
    }

    #[test]
    fn initial_lp_shares_reject_zero_amounts() {
        let res = calculate_lp_shares(0, 30, 0, 0, 0);
        assert!(res.is_err());
    }

    #[test]
    fn subsequent_lp_shares_use_min_ratio() {
        let shares = calculate_lp_shares(50, 80, 100, 100, 1000).unwrap();
        assert_eq!(shares, 500);
    }

    #[test]
    fn subsequent_lp_shares_require_nonzero_reserves() {
        let res = calculate_lp_shares(50, 80, 0, 100, 1000);
        assert!(res.is_err());
    }

    #[test]
    fn swap_output_applies_fee_split_and_floor_rounding() {
        let (amount_out, protocol_fee, lp_fee) =
            calculate_swap_output(1000, 10_000, 10_000, 30, 20).unwrap();

        assert_eq!(protocol_fee, 2);
        assert_eq!(lp_fee, 3);
        assert_eq!(amount_out, 904);
    }

    #[test]
    fn swap_output_rejects_when_tradeable_becomes_zero() {
        let res = calculate_swap_output(1, 10_000, 10_000, 5000, 5000);
        assert!(res.is_err());
    }

    #[test]
    fn remove_liquidity_is_proportional_and_floored() {
        let (amount_a, amount_b) = calculate_remove_liquidity(333, 1000, 1000, 2000).unwrap();
        assert_eq!(amount_a, 333);
        assert_eq!(amount_b, 666);
    }

    #[test]
    fn remove_liquidity_rejects_lp_amount_over_supply() {
        let res = calculate_remove_liquidity(1001, 1000, 1000, 1000);
        assert!(res.is_err());
    }

    #[test]
    fn update_twap_no_elapsed_time_is_noop() {
        let (price_a, price_b, ts) = update_twap(100, 200, 10, 50, 100, 10).unwrap();
        assert_eq!(price_a, 100);
        assert_eq!(price_b, 200);
        assert_eq!(ts, 10);
    }

    #[test]
    fn update_twap_accumulates_q32_price_over_time() {
        let (price_a, price_b, ts) = update_twap(0, 0, 10, 200, 100, 15).unwrap();
        assert_eq!(price_a, 10_737_418_240);
        assert_eq!(price_b, 42_949_672_960);
        assert_eq!(ts, 15);
    }

    #[test]
    fn update_twap_rejects_zero_reserves() {
        let res = update_twap(0, 0, 10, 0, 100, 20);
        assert!(res.is_err());
    }
}