# Exia AMM Design Document

This document is the protocol design baseline for the Exia constant-product AMM implemented in Anchor.

## 1. Scope and Objectives

### Objectives

- Implement a constant-product AMM on Solana with the invariant $x \cdot y = k$.
- Support pool initialization, liquidity add/remove, and token swaps.
- Enforce deterministic PDA ownership for state, vaults, and LP mint authority.
- Enforce slippage protection and bounded fee configuration.
- Keep arithmetic integer-only with checked operations.

### Non-objectives (current version)

- Multi-pool factory registry.
- External oracle integration consumers.
- Concentrated liquidity or dynamic fee curves.

## 2. Program Architecture

The program is organized into four focused layers.

- State model: `programs/exia_amm/src/state.rs`
- Instruction account constraints: `programs/exia_amm/src/instructions.rs`
- Math primitives: `programs/exia_amm/src/math.rs`
- Handlers and CPI wiring: `programs/exia_amm/src/lib.rs`

This split keeps business math testable and minimizes accidental coupling between account validation and numerical logic.

## 3. Account Model

The primary protocol account is `PoolState`, which stores:

- Token mints (`token_a_mint`, `token_b_mint`)
- Vault addresses (`token_a_vault`, `token_b_vault`)
- LP mint address (`lp_mint`)
- Fee config (`lp_fee_bps`, `protocol_fee_bps`)
- Authority and safety controls (`authority`, `pending_authority`, `is_paused`)
- Invariant and oracle telemetry (`k_last`, TWAP accumulators, timestamp)

`PoolState` is canonical under seeds:

- `[b"pool", token_a_mint, token_b_mint]`

Vaults and LP mint are also canonical PDAs derived from `pool_state`.

See [docs/architecture-account-diagram.md](docs/architecture-account-diagram.md) for the full PDA graph and account diagram.

## 4. Instruction Surface

Core user instructions:

- `initialize_pool`
- `add_liquidity`
- `swap`
- `remove_liquidity`

Admin instructions:

- `update_fees`
- `set_paused`
- `rotate_treasury`
- `propose_authority`
- `accept_authority`

Detailed account and constraint-level behavior is documented in [docs/instruction-account-reference.md](docs/instruction-account-reference.md).

## 5. Invariant and Pricing Design

The execution model is Uniswap-v2 style CPMM with integer arithmetic:

- Fees are removed from input before curve evaluation.
- LP issuance uses floor-sqrt on first mint and min-ratio on subsequent mints.
- Rounding is intentionally floor-biased to avoid value leakage from pool reserves.

Formal equations and edge-case policies are in [docs/math-and-invariant-writeup.md](docs/math-and-invariant-writeup.md).

## 6. Security Design Choices

### Constraint-first account safety

Critical authority checks are embedded at the account layer with:

- `seeds` and `bump` for canonical PDA validation
- `has_one = authority` for admin authorization
- `address = ...` for vault and mint pinning
- mint constraints on treasury rotation

### Runtime safety checks

- Fee upper bounds (`<= 500 bps`) at init and update.
- Slippage guard (`minimum_amount_out`) in `swap`.
- Pool pause gate for user actions that alter reserves.
- Mint-direction checks for user token accounts and treasury account.

### Arithmetic discipline

- `checked_mul`, `checked_add`, `checked_sub`, `checked_div`.
- Widened intermediate math to `u128`.
- Explicit error on overflow or invalid liquidity states.

## 7. Tradeoffs and Rationale

- Chosen: simple single-pool architecture per mint pair.
- Benefit: lower complexity and easier auditability for a course-grade major project.
- Cost: no global factory indexing or shared protocol config.

- Chosen: explicit protocol fee vaults outside pool vaults.
- Benefit: transparent fee accounting and treasury isolation.
- Cost: slightly larger account surface in swap paths.

- Chosen: TWAP accumulation in pool state.
- Benefit: future oracle composability without redesigning storage layout.
- Cost: additional state writes in liquidity and swap paths.

## 8. Verification Strategy

The repository validates correctness at two levels.

- Pure math unit tests for formula and rounding behavior.
- Integration tests with LiteSVM for instruction paths, authority rules, pause/slippage checks, and adversarial cases.

See [docs/security-testing-guide.md](docs/security-testing-guide.md) for the complete test matrix and execution commands.

## 9. Deployment Targets

- Localnet and devnet configuration are both present in `Anchor.toml`.
- Program ID is fixed via `declare_id!` and mirrored in Anchor config.

Deployment runbook is in `README.md`.