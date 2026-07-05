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
    classDef signer fill:#888780,stroke:#2C2C2A,color:#fff,font-weight:bold
    classDef mint fill:#7F77DD,stroke:#26215C,color:#fff
    classDef pda fill:#1D9E75,stroke:#04342C,color:#fff
    classDef user fill:#378ADD,stroke:#042C53,color:#fff
    classDef treasury fill:#D85A30,stroke:#4A1B0C,color:#fff

    User["👤 User Signer"]:::signer
    Admin["🔑 Authority Signer"]:::signer

    subgraph Mints [" Token Mints "]
        TA["Token A Mint"]:::mint
        TB["Token B Mint"]:::mint
    end

    subgraph Pool [" Pool — Program-Owned PDAs "]
        PS["PoolState"]:::pda
        VA["Vault A"]:::pda
        VB["Vault B"]:::pda
        LP["LP Mint"]:::pda
    end

    subgraph UserAccts [" User Token Accounts "]
        UTA["User Token A"]:::user
        UTB["User Token B"]:::user
        ULP["User LP Token"]:::user
    end

    subgraph Treasury [" Protocol Treasury "]
        TRA["Treasury Token A"]:::treasury
        TRB["Treasury Token B"]:::treasury
    end

    %% Ownership / mint links
    TA --> VA
    TB --> VB
    PS -.-> VA
    PS -.-> VB
    PS -.-> LP
    User -.-> UTA
    User -.-> UTB
    User -.-> ULP

    %% Liquidity
    UTA -->|add_liquidity| VA
    UTB -->|add_liquidity| VB
    LP -->|mint_to| ULP
    VA -->|remove_liquidity| UTA
    VB -->|remove_liquidity| UTB
    ULP -->|burn| LP

    %% Swaps
    UTA -->|"swap A→B (in)"| VA
    VB -->|"swap A→B (out)"| UTB
    UTB -->|"swap B→A (in)"| VB
    VA -->|"swap B→A (out)"| UTA

    %% Fees
    VA -->|protocol fee| TRA
    VB -->|protocol fee| TRB

    %% Admin
    Admin -.->|"update_fees / set_paused /<br/>rotate_treasury / propose_authority"| PS

    linkStyle 2,3,4,5,6,7 stroke:#888780,stroke-dasharray: 4 3
    linkStyle 20 stroke:#D85A30,stroke-width:1px
    linkStyle 21 stroke:#D85A30,stroke-width:1px
    linkStyle 22 stroke:#534AB7,stroke-dasharray: 4 3,stroke-width:1.5px
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
