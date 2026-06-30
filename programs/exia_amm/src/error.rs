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
}
