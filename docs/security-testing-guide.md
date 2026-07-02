# Security and Testing Guide

This supplemental document provides a practical checklist for audit review and viva demonstrations.

## 1. Security Checklist

### Account and authority controls

- Canonical PDA checks (`seeds`, `bump`) on pool state.
- Vault and LP mint address pinning to `pool_state` fields.
- Admin actions protected with `has_one = authority`.
- Two-step authority transfer (`propose_authority` + `accept_authority`).

### Swap path safety

- Directional mint checks for `user_token_in` and `user_token_out`.
- Directional treasury account and mint checks.
- Slippage check via `minimum_amount_out`.

### Arithmetic safety

- Checked arithmetic for all reserve, fee, and share operations.
- Integer-only execution (no floating point).
- Error returns for invalid and underflow/overflow states.

### Initialization and pause controls

- `#[account(init)]` prevents reinitialization of existing accounts.
- Pause flag disables high-impact user actions.

## 2. Current Test Matrix

Integration coverage in `programs/exia_amm/tests/test_initialize.rs` includes:

- Pool initialization.
- Add liquidity and LP mint behavior.
- Swap A->B and B->A.
- Remove liquidity.
- TWAP accumulation behavior.
- Treasury direction enforcement.
- Excessive fee rejection.
- Authority transfer and old-authority rejection.
- Pause and slippage protection.
- Reinitialization rejection.
- Unauthorized admin action rejection.
- Accept-authority without pending proposal rejection.
- Wrong input-mint for swap direction rejection.

Unit coverage in `programs/exia_amm/src/math.rs` includes:

- LP share formulas.
- Swap output and fee split.
- Remove-liquidity formulas.
- TWAP edge cases.

## 3. How to Run Verification

```bash
# Unit + integration tests
cargo test --all

# Lint gate
cargo clippy --all-targets --all-features -- -D warnings

# Build docs for viva walkthrough
cargo doc --no-deps --document-private-items
```

## 4. Manual Adversarial Scenarios for Demo

- Attempt admin calls from a non-authority signer.
- Attempt swap with mismatched input mint and direction.
- Attempt swap with impossible `minimum_amount_out`.
- Attempt reinitialization of same pool PDA.
- Attempt accept-authority without prior proposal.

## 5. Suggested Next Hardening (Optional)

- Property-based tests on invariant monotonic behavior under random swap sequences.
- Dedicated failure tests for each error code path.
- Differential tests against a reference CPMM calculator.
- Compute unit profiling and stack usage snapshots for heavier scenarios.
