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

## Codebase Structure

The project strictly separates state modeling, financial mathematics, and execution handlers.

| Directory | File | Purpose |
| :--- | :--- | :--- |
| `src/` | `state.rs` | Data modeling, space allocation, and PDA structural definitions. |
| `src/` | `math.rs` | Pure mathematical execution (LP shares, swap output, fixed-point TWAP). |
| `src/` | `instructions.rs` | Anchor context validators, CPI seed derivation, and account constraint mapping. |
| `src/` | `error.rs` | Protocol-specific error definitions. |
| `src/` | `lib.rs` | Instruction handlers mapping inputs to state mutations via CPIs. |
| `tests/` | `test_initialize.rs` | `LiteSVM` integration suite covering invariants, MEV tests, and authority limits. |

## Local Development

**Prerequisites:**
* Rust 1.75.0+
* Solana CLI 1.18.0+
* Anchor CLI 0.29.0+

**Build & Test:**
```bash
# Install dependencies
npm install

# Compile the BPF program
anchor build

# Run integer safety and syntax checks
cargo clippy --all-targets --all-features -- -D warnings

# Execute the local LiteSVM test suite
cargo test
```

## Deployment

This repository is configured for both localnet and devnet in [Anchor.toml](Anchor.toml).

### 1) Prepare Solana CLI

```bash
# Confirm wallet path used by Anchor
solana address

# Check current cluster
solana config get
```

### 2) Build artifacts

```bash
anchor build
```

### 3) Deploy to localnet

```bash
solana config set --url http://127.0.0.1:8899
anchor deploy
```

### 4) Deploy to devnet

```bash
solana config set --url https://api.devnet.solana.com
solana airdrop 2
anchor deploy --provider.cluster devnet
```

### 5) Verify deployment

```bash
# Program id from Anchor.toml and declare_id! in the program
solana program show 2Hy7ouFwJLkG7cpAoSR4hGaFk3zPH2gAYLKjTMdGsqQs

# Optional: fetch generated IDL after build/deploy
ls -la target/idl/
```

## Notes

- The on-chain program id is fixed in [programs/exia_amm/src/lib.rs](programs/exia_amm/src/lib.rs) and mirrored in [Anchor.toml](Anchor.toml).
- If deploying with a different keypair, update both locations so they stay in sync.

## Documentation Index

This repository includes a complete major-project documentation set.

- Design document: [architecture.md](architecture.md)
- Architecture and account diagram: [docs/architecture-account-diagram.md](docs/architecture-account-diagram.md)
- Instruction and account reference: [docs/instruction-account-reference.md](docs/instruction-account-reference.md)
- Math and invariant writeup: [docs/math-and-invariant-writeup.md](docs/math-and-invariant-writeup.md)
- Supplemental security and testing guide: [docs/security-testing-guide.md](docs/security-testing-guide.md)

## Client SDK

- TypeScript SDK package: [app/README.md](app/README.md)