use anchor_lang::prelude::*;

pub const MINIMUM_LIQUIDITY: u64 = 1000;

pub fn calculate_lp_shares(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    total_lp_supply: u64,
) -> Result<u64> {

    // Scenario: First Liquidity Deposit
    if total_lp_supply == 0 {
        // sqrt(a * b)
        let product = (amount_a as u128).checked_mul(amount_b as u128)
            .ok_or(ErrorCode::MathOverflow)?;
        let initial_shares = (product as f64).sqrt() as u64;

        // Safety lock: Reject if the pool is too small
        if initial_shares < MINIMUM_LIQUIDITY {
            return Err(ErrorCode::InsufficientLiquidity.into());
        }

        Ok(initial_shares.checked_sub(MINIMUM_LIQUIDITY).unwrap())
    }
    // Scenario: Subsequent Deposit
    else {
        let share_a = (amount_a as u128).checked_mul(total_lp_supply as u128).unwrap()
            .checked_div(reserve_a as u128).unwrap();

        let share_b = (amount_b as u128).checked_mul(total_lp_supply as u128).unwrap()
            .checked_div(reserve_b as u128).unwrap();

        Ok(std::cmp::min(share_a, share_b) as u64)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Math operation overflowed")]
    MathOverflow,
    #[msg("Insufficient initial liquidity")]
    InsufficientLiquidity,
}