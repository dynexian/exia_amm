use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Math operation overflowed")]
    MathOverflow,
    #[msg("Insufficient initial liquidity")]
    InsufficientLiquidity,
    #[msg("Slippage tolerance exceeded: output below minimum")]
    SlippageExceeded,
    #[msg("Pool has no liquidity")]
    NoLiquidity,
    #[msg("Pool is paused")]
    PoolPaused,
    #[msg("Unauthorized: caller is not the pool authority")]
    Unauthorized,
    #[msg("Fee exceeds maximum allowed (500 bps)")]
    FeeTooHigh,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Token account does not match the pool direction or mint")]
    InvalidTokenAccount,
    #[msg("Treasury token account does not match the pool direction")]
    InvalidTreasury,
}
