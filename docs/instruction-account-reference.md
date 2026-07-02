# Instruction and Account Reference

This document is a handler-by-handler reference for instruction arguments, required accounts, and constraints.

## 1. initialize_pool

Purpose:

- Create a new pool for a token pair and initialize immutable routing fields.

Arguments:

- `lp_fee_bps: u16`
- `protocol_fee_bps: u16`
- `authority: Pubkey`

Behavior:

- Validates fee caps (`<= 500 bps`).
- Initializes `pool_state`, `vault_a`, `vault_b`, and `lp_mint` PDAs.
- Stores treasury accounts and authority.
- Initializes TWAP and invariant telemetry fields.

Required accounts:

- `payer` signer and fee payer.
- `pool_state` PDA (`init`, canonical seeds).
- `token_a_mint`, `token_b_mint`.
- `treasury_token_a`, `treasury_token_b` with matching mints.
- `vault_a`, `vault_b` token accounts (`init`, PDA-owned by `pool_state`).
- `lp_mint` PDA mint (`init`, authority is `pool_state`).
- `system_program`, `token_program`, `rent`.

## 2. add_liquidity

Purpose:

- Deposit both assets and mint proportional LP shares.

Arguments:

- `amount_a: u64`
- `amount_b: u64`

Behavior:

- Rejects if pool is paused.
- Syncs TWAP with pre-mutation reserves.
- Calculates shares from pre-deposit reserves.
- Transfers token A/B from user into vaults.
- Mints LP tokens to user account.
- Updates `k_last` from post-operation vault balances.

Required accounts:

- `user` signer.
- `pool_state` PDA.
- `user_token_a`, `user_token_b`, `user_lp_token` (mutable).
- `vault_a`, `vault_b` pinned to pool state addresses.
- `lp_mint` pinned to pool state.
- `token_program`, `system_program`.

## 3. swap

Purpose:

- Swap exact input amount against CPMM curve with minimum output slippage protection.

Arguments:

- `amount_in: u64`
- `minimum_amount_out: u64`
- `a_to_b: bool`

Behavior:

- Rejects if pool is paused.
- Validates input/output user token mints against direction.
- Validates direction-matching treasury account.
- Calculates `(amount_out, protocol_fee, lp_fee)` from reserves and fee config.
- Enforces `amount_out >= minimum_amount_out`.
- Syncs TWAP before reserve mutation.
- Transfers input from user to pool vault.
- Transfers protocol fee from input-side vault to treasury.
- Transfers output from opposite vault to user.
- Updates `k_last`.

Required accounts:

- `user` signer.
- `pool_state` PDA.
- `user_token_in`, `user_token_out`.
- `vault_a`, `vault_b` pinned to pool state.
- `treasury_token_in` direction-specific protocol fee receiver.
- `token_program`.

## 4. remove_liquidity

Purpose:

- Burn LP shares and redeem proportional token A and token B reserves.

Arguments:

- `lp_amount: u64`

Behavior:

- Syncs TWAP before reserve mutation.
- Computes output amounts proportionally to LP supply.
- Burns user LP tokens.
- Transfers both underlying tokens from vaults to user.
- Updates `k_last`.

Required accounts:

- `user` signer.
- `pool_state` PDA.
- `user_token_a`, `user_token_b`, `user_lp_token`.
- `vault_a`, `vault_b`, `lp_mint` pinned to pool state.
- `token_program`.

## 5. update_fees (admin)

Purpose:

- Update LP and protocol fee basis points.

Arguments:

- `new_lp_fee_bps: u16`
- `new_protocol_fee_bps: u16`

Behavior:

- Requires current authority signer via `has_one`.
- Enforces each fee `<= 500` bps.

Required accounts:

- `authority` signer.
- `pool_state` PDA with `has_one = authority`.

## 6. set_paused (admin)

Purpose:

- Toggle emergency pause flag.

Arguments:

- `paused: bool`

Behavior:

- Requires current authority signer via `has_one`.
- Updates `pool_state.is_paused`.

Required accounts:

- `authority` signer.
- `pool_state` PDA with `has_one = authority`.

## 7. rotate_treasury (admin)

Purpose:

- Rotate protocol fee destination token accounts.

Arguments:

- none

Behavior:

- Requires authority signer via `has_one`.
- Validates replacement treasury account mints.
- Writes new treasury account addresses to pool state.

Required accounts:

- `authority` signer.
- `pool_state` PDA with `has_one = authority`.
- `new_treasury_token_a` with mint = token A mint.
- `new_treasury_token_b` with mint = token B mint.

## 8. propose_authority (admin)

Purpose:

- Start two-step authority transfer.

Arguments:

- `new_authority: Pubkey`

Behavior:

- Requires current authority signer via `has_one`.
- Sets `pending_authority`.

Required accounts:

- `authority` signer.
- `pool_state` PDA with `has_one = authority`.

## 9. accept_authority

Purpose:

- Finalize authority transfer by pending authority signer.

Arguments:

- none

Behavior:

- Requires `new_authority` signer.
- Checks signer equals `pending_authority`.
- Sets `authority = new_authority` and clears pending field.

Required accounts:

- `new_authority` signer.
- `pool_state` PDA.

## Error Mapping

Primary protocol errors:

- `MathOverflow`
- `SlippageExceeded`
- `InsufficientLiquidity`
- `NoLiquidity`
- `PoolPaused`
- `Unauthorized`
- `FeeTooHigh`
- `InvalidAmount`
- `InvalidTokenAccount`
- `InvalidTreasury`
