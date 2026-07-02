use anchor_lang::prelude::*;

/// Custom error codes thrown by the Exia AMM program during state validation or execution failure.
#[error_code]
pub enum ErrorCode {
    /// Triggered if an unchecked multiplication or addition would wrap around.
    #[msg("Math operation overflowed")]
    MathOverflow,
    
    /// Triggered when the initial liquidity deposit generates zero LP shares.
    #[msg("Insufficient initial liquidity")]
    SlippageExceeded,
    
    /// Triggered if the computed output token amount falls below the user's `minimum_amount_out` threshold.
    #[msg("Slippage tolerance exceeded: output below minimum")]
    InsufficientLiquidity,
    
    /// Triggered when an operation is attempted on an AMM vault that has zero active reserves.
    #[msg("Pool has no liquidity")]
    NoLiquidity,
    
    /// Triggered when a trader or liquidity provider attempts an execution while `is_paused` is set to true.
    #[msg("Pool is paused")]
    PoolPaused,
    
    /// Triggered if a non-authority entity calls a privileged administrative instruction.
    #[msg("Unauthorized: caller is not the pool authority")]
    Unauthorized,
    
    /// Triggered if an admin attempts to update the fee metrics past the maximum allowable cap (500 bps / 5%).
    #[msg("Fee exceeds maximum allowed (500 bps)")]
    FeeTooHigh,
    
    /// Triggered when an instruction payload passes an input value equal to zero.
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    
    /// Triggered if a user's token account mint does not align cryptographically with the pool's designated underlying mint keys.
    #[msg("Token account does not match the pool direction or mint")]
    InvalidTokenAccount,
    
    /// Triggered if the target destination account for protocol revenue collects the wrong token type during a swap execution.
    #[msg("Treasury token account does not match the pool direction")]
    InvalidTreasury,
}