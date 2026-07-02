# Exia AMM

A constant-product Automated Market Maker (CPMM) built on the Solana Virtual Machine using the Anchor framework.

Exia implements a decentralized exchange primitive enforcing the $x \cdot y = k$ invariant. It features sovereign Program Derived Address (PDA) vaults, strict integer arithmetic, dual protocol fee treasuries, and a Time-Weighted Average Price (TWAP) oracle.

## Features

* **Constant-Product Swap Math:** Trades execute against the curve with fees extracted prior to state mutation.
* **Proportional LP Minting:** Liquidity providers receive LP tokens representing their fractional ownership of the pool reserves.
* **Cryptographic Sovereignty:** All pool vaults and LP mints are deterministic PDAs. The program retains absolute signing authority over reserves.
* **MEV & Slippage Protection:** Swaps enforce a strictly validated minimum output bound to prevent sandwich attacks.
* **Protocol & LP Fee Split:** Distinct fee tiers route LP yields back into the pool while sending protocol revenue to dedicated external treasury accounts.
* **TWAP Accumulator:** On-chain price oracle utilizing Q32.32 fixed-point math for safe external protocol composability.
* **Protocol Hardening:** Authority system, emergency pause, fee update caps, treasury rotation, and two-step authority transfer.

## Codebase Structure

| Directory | File | Purpose |
| :--- | :--- | :--- |
| `programs/exia_amm/src/` | `state.rs` | Data modeling, space allocation, and PDA structural definitions. |
| `programs/exia_amm/src/` | `math.rs` | Pure mathematical execution (LP shares, swap output, fixed-point TWAP). |
| `programs/exia_amm/src/` | `instructions.rs` | Anchor context validators, CPI seed derivation, and account constraint mapping. |
| `programs/exia_amm/src/` | `error.rs` | Protocol-specific error definitions. |
| `programs/exia_amm/src/` | `lib.rs` | Instruction handlers mapping inputs to state mutations via CPIs. |
| `programs/exia_amm/tests/` | `test_initialize.rs` | LiteSVM integration suite — 15 tests covering invariants, MEV, and authority limits. |
| `app/src/` | `index.ts` | TypeScript SDK — `ExiaAmmClient` wrapping all 8 instructions. |
| `app/examples/` | `init.ts` | Pool initialization script. |
| `app/examples/` | `add_liquidity.ts` | Liquidity deposit script. |
| `app/examples/` | `swap.ts` | Swap execution script. |
| `app/examples/` | `remove_liquidity.ts` | Liquidity withdrawal script. |
| `app/examples/` | `rotate_treasury.ts` | Treasury rotation admin script. |

## Instructions

| Instruction | Description |
| :--- | :--- |
| `initialize_pool` | Creates pool state, vaults, and LP mint in a single atomic transaction. |
| `add_liquidity` | Deposits Token A and Token B, mints proportional LP shares. |
| `swap` | Executes a trade with 4-step fee split and slippage protection. |
| `remove_liquidity` | Burns LP tokens, returns proportional reserves to the user. |
| `update_fees` | Admin: updates LP and protocol fee rates (max 500 bps each). |
| `set_paused` | Admin: emergency kill-switch blocking swaps and deposits. |
| `rotate_treasury` | Admin: redirects protocol fee collection to new token accounts. |
| `propose_authority` | Admin: nominates a new pool authority (step 1 of 2). |
| `accept_authority` | Nominated wallet accepts authority (step 2 of 2). |

## Local Development

**Prerequisites:**
* Rust (stable or nightly)
* Solana CLI 4.x (Agave)
* Anchor CLI 1.1.2
* Node.js 18+

**Build & Test:**
```bash
# Compile the BPF program
anchor build

# Run the full test suite (27 tests: 12 unit + 15 integration)
cargo test

# Type-check the TypeScript SDK
npm --prefix app run typecheck
```

## Running Scripts

All scripts are driven by environment variables:

```bash
export ANCHOR_WALLET=~/.config/solana/id.json
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export TOKEN_A_MINT=<mint-a-pubkey>
export TOKEN_B_MINT=<mint-b-pubkey>
```

**Initialize pool:**
```bash
npm --prefix app run init
```

**Add liquidity:**
```bash
export AMOUNT_A=100
export AMOUNT_B=100
npm --prefix app run add-liquidity
```

**Swap:**
```bash
export AMOUNT_IN=10
export MIN_OUT=9
export DIRECTION=AtoB   # or BtoA
export TREASURY_A=<treasury-token-a-account>
export TREASURY_B=<treasury-token-b-account>
npm --prefix app run swap
```

**Remove liquidity:**
```bash
export LP_AMOUNT=10
npm --prefix app run remove-liquidity
```

**Rotate treasury:**
```bash
export NEW_TREASURY_A=<new-token-a-account>
export NEW_TREASURY_B=<new-token-b-account>
npm --prefix app run rotate-treasury
```

## Deployment

```bash
# Build artifacts
anchor build

# Deploy to devnet
solana config set --url https://api.devnet.solana.com
solana airdrop 2
anchor deploy --provider.cluster devnet

# Sync program ID if keypair changed
anchor keys sync
anchor build
```

## Devnet Deployment

| | |
| :--- | :--- |
| **Program ID** | `2Hy7ouFwJLkG7cpAoSR4hGaFk3zPH2gAYLKjTMdGsqQs` |
| **Pool State** | `B72fpafN3c4JZfjuXB53ZfDoRYkgnx5bNTJt8WaQtb2U` |
| **Vault A** | `5ZV1NWDs8UVHKT9gnBb8n5gGnxbg2XXjs4FHKcyxHaPR` |
| **Vault B** | `DwUHtjHx9Lk6W3xmDRNd5qJV2PwpWhFB9a3ZLDZfgj8s` |
| **LP Mint** | `AsxhqxddiPHjnqdfAXEcJFXdXnn86sqs3AePAG87pWRY` |
| **Token A** | `4kSPiLkhPzfoPb23zadFy1twDaLmHpDwyJ7B9AFiG18m` |
| **Token B** | `57NU9Y4esMQndM7TcqNeADRmV3Ab4XrqejsXz1Vb2sia` |
| **LP Fee** | 25 bps (0.25%) |
| **Protocol Fee** | 5 bps (0.05%) |
| **Authority** | `HU5qbCMApC3ULAEa8To1xdRHEMQ8n9vYCvNqo5zzHiJg` |

**Confirmed on-chain transactions:**

| Event | Signature |
| :--- | :--- |
| Pool initialization | `4Gjg59EgUF36rnbSSwMWTHnfxTEDhDwVne1pWkLMSdVuZZ18YUmCAUeRitfae9JnRs5oywZLygPQfasL3RQSWudM` |
| Admin write proof (updateFees) | `2FLLns7qTZjmokNXQyS8Y5icwNi3WyYbnFgAsWp7Vq2XKJpEZq3pZTutvAWR8nYNjJTiFEMYrLtBskC6NMEuUjgt` |
| Add liquidity (100/100) | `2tKTYPQjexoLRn799qWcDiUDh5Q6nJEeDq4MSQxW8ebVUKchAq8A7GBnxhFmgZijkm4MHj2MgHG3DuhvdHupvCfn` |
| Treasury rotation | `42SUGNf4QvtqCuCrBceVZ3UcRorQpDDWck24wVmN3kRe6n7XWAwfwUy384SNbSwywfXaF5UxR1sUv5j1tkBdCKEP` |
| First swap (10 A → 9.066 B) | `368KZyXvyWUwNZRxcBEiXww1DqeqyWTGxsvXN1ByeJk7j88JjrL2wrXGNCstwGc6GZpUwVK6fLb97D1tZBkrGXmA` |
| Remove liquidity (10 LP) | `HxDawi8seJJqWRHwEFHAWTkQ9WN864vyfP2i8muM9YQ2TFc6Z3a2PiUqdyUxVH2XMYy2U6ML1DKCxzuW7jwVbdP` |

## Documentation Index

- Architecture design: [architecture.md](architecture.md)
- Account diagram: [docs/architecture-account-diagram.md](docs/architecture-account-diagram.md)
- Instruction reference: [docs/instruction-account-reference.md](docs/instruction-account-reference.md)
- Math writeup: [docs/math-and-invariant-writeup.md](docs/math-and-invariant-writeup.md)
- Security and testing guide: [docs/security-testing-guide.md](docs/security-testing-guide.md)
- TypeScript SDK: [app/README.md](app/README.md)
