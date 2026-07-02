# Exia AMM TypeScript SDK

This package provides a typed TypeScript client for the on-chain Exia AMM program.

## What it includes

- Canonical PDA derivation helpers.
- Typed wrappers around all program instructions.
- Utility to load a local keypair.
- Example script for pool address derivation and state fetch.

## Install

From repository root:

```bash
cd app
npm install
```

## Type-check

```bash
npm run typecheck
```

## Example usage

```bash
export RPC_URL=http://127.0.0.1:8899
export TOKEN_A_MINT=<token-a-mint-pubkey>
export TOKEN_B_MINT=<token-b-mint-pubkey>
npm run example
```

## Write transaction proof demo (admin)

This demo sends a real transaction using `updateFees`, but uses the current on-chain fee values so behavior is state-preserving.

```bash
export RPC_URL=http://127.0.0.1:8899
export TOKEN_A_MINT=<token-a-mint-pubkey>
export TOKEN_B_MINT=<token-b-mint-pubkey>
export WRITE_DEMO_CONFIRM=YES_I_KNOW_THIS_SENDS_A_TX
npm run example:write
```

Notes:

- Your wallet must be the pool authority, otherwise the transaction will fail with an authorization error.
- On localnet, ignore the explorer URL printed by the script.

## SDK entry points

- `ExiaAmmClient`: main instruction wrapper.
- `derivePoolPdas`: deterministic PDA helper.
- `EXIA_AMM_PROGRAM_ID`: default program id from IDL.
- `keypairFromFile`: read keypair from JSON secret key file.

## Minimal integration snippet

```ts
import { AnchorProvider } from "@coral-xyz/anchor";
import { ExiaAmmClient } from "./src/index.js";

const provider = AnchorProvider.env();
const client = ExiaAmmClient.fromProvider(provider);
```

The client wraps all instructions currently exposed by the program:

- `initializePool`
- `addLiquidity`
- `swap`
- `removeLiquidity`
- `updateFees`
- `setPaused`
- `rotateTreasury`
- `proposeAuthority`
- `acceptAuthority`
