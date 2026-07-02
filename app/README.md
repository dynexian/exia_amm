# Exia AMM TypeScript SDK

Typed TypeScript client for the on-chain Exia AMM program.

## What it includes

- Canonical PDA derivation helpers.
- `ExiaAmmClient` — typed wrappers around all 9 program instructions.
- Utility to load a local keypair from a JSON secret key file.
- Example scripts for every protocol operation.

## Install

```bash
cd app
npm install
```

## Type-check

```bash
npm run typecheck
```

## Available scripts

| Script | Command | Description |
| :--- | :--- | :--- |
| Read demo | `npm run example` | Fetches and prints pool state. No transaction sent. |
| Write demo | `npm run example:write` | Sends a state-preserving `updateFees` transaction as admin proof. |
| Init pool | `npm run init` | Initializes a new pool with treasury ATAs and authority. |
| Add liquidity | `npm run add-liquidity` | Deposits tokens and receives LP shares. |
| Swap | `npm run swap` | Executes a directional swap with slippage protection. |
| Remove liquidity | `npm run remove-liquidity` | Burns LP tokens and reclaims proportional reserves. |
| Rotate treasury | `npm run rotate-treasury` | Admin: redirects protocol fee accounts. |

## Environment variables

All scripts share these common variables:

```bash
export ANCHOR_WALLET=~/.config/solana/id.json
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export TOKEN_A_MINT=<mint-pubkey>
export TOKEN_B_MINT=<mint-pubkey>
```

Script-specific variables:

| Script | Variable | Default | Description |
| :--- | :--- | :--- | :--- |
| `add-liquidity` | `AMOUNT_A` | `100` | Token A amount to deposit |
| `add-liquidity` | `AMOUNT_B` | `100` | Token B amount to deposit |
| `swap` | `AMOUNT_IN` | `10` | Input token amount |
| `swap` | `MIN_OUT` | `1` | Minimum output (slippage guard) |
| `swap` | `DIRECTION` | `AtoB` | `AtoB` or `BtoA` |
| `swap` | `TREASURY_A` | required | Treasury token account for Token A fees |
| `swap` | `TREASURY_B` | required | Treasury token account for Token B fees |
| `remove-liquidity` | `LP_AMOUNT` | `10` | LP tokens to burn |
| `rotate-treasury` | `NEW_TREASURY_A` | required | New Token A fee destination |
| `rotate-treasury` | `NEW_TREASURY_B` | required | New Token B fee destination |

## SDK entry points

```ts
import { ExiaAmmClient, derivePoolPdas, EXIA_AMM_PROGRAM_ID, keypairFromFile } from "./src/index";
```

- `ExiaAmmClient` — main instruction wrapper class
- `derivePoolPdas` — deterministic PDA derivation for pool, vaults, and LP mint
- `EXIA_AMM_PROGRAM_ID` — default program ID from IDL
- `keypairFromFile` — reads a Solana keypair from a JSON secret key file

## Minimal integration

```ts
import { AnchorProvider } from "@coral-xyz/anchor";
import { ExiaAmmClient } from "./src/index";

const provider = AnchorProvider.env();
const client = ExiaAmmClient.fromProvider(provider);

// Fetch pool state
const pool = await client.fetchPoolState(mintA, mintB);

// Execute a swap
const sig = await client.swap({
    user: wallet.publicKey,
    tokenAMint: mintA,
    tokenBMint: mintB,
    userTokenIn,
    userTokenOut,
    treasuryTokenIn,
    amountIn: 1_000_000_000n,
    minimumAmountOut: 900_000_000n,
    aToB: true,
});
```

## Instructions wrapped

| Method | On-chain instruction |
| :--- | :--- |
| `initializePool` | `initialize_pool` |
| `addLiquidity` | `add_liquidity` |
| `swap` | `swap` |
| `removeLiquidity` | `remove_liquidity` |
| `updateFees` | `update_fees` |
| `setPaused` | `set_paused` |
| `rotateTreasury` | `rotate_treasury` |
| `proposeAuthority` | `propose_authority` |
| `acceptAuthority` | `accept_authority` |
