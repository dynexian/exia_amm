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
---
config:
  layout: elk
---

flowchart TD
    %% === Class Definitions ===
    classDef signer fill:#eef2ff,stroke:#818cf8,color:#1e1b4b,font-weight:bold
    classDef mint fill:#f0fdfa,stroke:#2dd4bf,color:#064e3b
    classDef pda fill:#f5f3ff,stroke:#a78bfa,color:#4c1d95
    classDef user fill:#ecfeff,stroke:#22d3ee,color:#083344
    classDef treasury fill:#fff1f2,stroke:#fb7185,color:#7f1d1d

    %% === Actors ===
    User[User Signer]:::signer
    Admin[Authority Signer]:::signer

    %% === Token Mints ===
    subgraph Mints[Token Mints]
        direction TB
        TA[Token A Mint]:::mint
        TB[Token B Mint]:::mint
    end

    %% === Pool (Program Owned PDAs) ===
    subgraph Pool["Pool (Program-Owned PDAs)"]
        direction TB
        PS[Pool State]:::pda
        VA[Vault A]:::pda
        VB[Vault B]:::pda
        LP[LP Mint]:::pda
    end

    %% === User Token Accounts ===
    subgraph UserAccts[User Token Accounts]
        direction TB
        UTA[User Token A]:::user
        UTB[User Token B]:::user
        ULP[User LP Token]:::user
    end

    %% === Treasury ===
    subgraph Treasury[Protocol Treasury]
        direction TB
        TRA[Treasury Token A]:::treasury
        TRB[Treasury Token B]:::treasury
    end

    %% === Mint to Vault Connections ===
    TA --> VA
    TB --> VB

    %% === Pool Control Links ===
    PS -.-> VA
    PS -.-> VB
    PS -.-> LP

    %% === User ↔ Accounts ===
    User -.-> UTA
    User -.-> UTB
    User -.-> ULP

    %% === Liquidity Actions ===
    UTA -->|add_liquidity| VA
    UTB -->|add_liquidity| VB
    LP -->|mint_to| ULP

    VA -->|remove_liquidity| UTA
    VB -->|remove_liquidity| UTB
    ULP -->|burn| LP

    %% === Swap Routes ===
    UTA -->|"swap A → B in"| VA
    VB -->|"swap A → B out"| UTB
    UTB -->|"swap B → A in"| VB
    VA -->|"swap B → A out"| UTA

    %% === Protocol Fees ===
    VA -->|protocol fee| TRA
    VB -->|protocol fee| TRB
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
