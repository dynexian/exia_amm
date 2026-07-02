# Architecture and Account Diagram

This document explains how protocol accounts are derived, connected, and validated.

## 1. Program and PDA Graph

Program id:

- `2Hy7ouFwJLkG7cpAoSR4hGaFk3zPH2gAYLKjTMdGsqQs`

Canonical addresses:

- Pool state PDA: `[b"pool", token_a_mint, token_b_mint]`
- Vault A PDA: `[b"vault_a", pool_state]`
- Vault B PDA: `[b"vault_b", pool_state]`
- LP mint PDA: `[b"lp_mint", pool_state]`

## 2. Account Relationship Diagram

```mermaid
flowchart TD
    User[User Signer]
    Admin[Authority Signer]
    TA[Token A Mint]
    TB[Token B Mint]

    PS[PoolState PDA]
    VA[Vault A PDA TokenAccount]
    VB[Vault B PDA TokenAccount]
    LP[LP Mint PDA]

    UTA[User Token A]
    UTB[User Token B]
    ULP[User LP Token]

    TRA[Treasury Token A]
    TRB[Treasury Token B]

    TA --> VA
    TB --> VB
    PS --> VA
    PS --> VB
    PS --> LP

    User --> UTA
    User --> UTB
    User --> ULP

    UTA -->|add_liquidity| VA
    UTB -->|add_liquidity| VB
    LP -->|mint_to| ULP

    UTA -->|swap A->B in| VA
    VB -->|swap A->B out| UTB
    VA -->|protocol fee A| TRA

    UTB -->|swap B->A in| VB
    VA -->|swap B->A out| UTA
    VB -->|protocol fee B| TRB

    ULP -->|burn| LP
    VA -->|remove_liquidity| UTA
    VB -->|remove_liquidity| UTB

    Admin -->|update_fees/set_paused/rotate_treasury/propose_authority| PS
```

## 3. PoolState Layout

`PoolState::MAX_SPACE = 359 bytes` includes discriminator and all fields.

Key groups:

- Identity and routing: token mints, vault addresses, LP mint.
- Fee config: LP fee bps and protocol fee bps.
- Admin controls: authority and pause state.
- Observability: `k_last` and TWAP cumulative fields.

## 4. Constraint Mapping (Anchor)

Most safety checks are expressed through account constraints:

- `init` on pool state and PDA-owned token accounts prevents re-init overwrite.
- `seeds` and `bump` enforce canonical account derivation.
- `address = pool_state.<field>` pins vault/mint account routing.
- `has_one = authority` gates admin operations.
- Mint constraints on treasury rotation ensure token-type correctness.

## 5. Design Notes

- The `authority_bump` field is reserved for layout stability in future upgrades.
- TWAP fields are in core state to avoid migrations when adding oracle consumers later.
- Protocol treasuries are external token accounts for explicit fee custody.
