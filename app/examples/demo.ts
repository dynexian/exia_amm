import { AnchorProvider, Wallet } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { ExiaAmmClient, keypairFromFile } from "../src/index.js";

async function main() {
  const rpcUrl = process.env.RPC_URL ?? "http://127.0.0.1:8899";
  const walletPath = process.env.WALLET_PATH ?? `${process.env.HOME}/.config/solana/id.json`;

  const tokenAMintRaw = process.env.TOKEN_A_MINT;
  const tokenBMintRaw = process.env.TOKEN_B_MINT;

  if (!tokenAMintRaw || !tokenBMintRaw) {
    throw new Error("Set TOKEN_A_MINT and TOKEN_B_MINT environment variables.");
  }

  const tokenAMint = new PublicKey(tokenAMintRaw);
  const tokenBMint = new PublicKey(tokenBMintRaw);

  const signer = await keypairFromFile(walletPath);
  const wallet = new Wallet(signer);
  const connection = new Connection(rpcUrl, "confirmed");
  const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });

  const client = ExiaAmmClient.fromProvider(provider);
  const pdas = client.derivePoolPdas(tokenAMint, tokenBMint);

  console.log("Program:", client.program.programId.toBase58());
  console.log("Pool State:", pdas.poolState.toBase58());
  console.log("Vault A:", pdas.vaultA.toBase58());
  console.log("Vault B:", pdas.vaultB.toBase58());
  console.log("LP Mint:", pdas.lpMint.toBase58());

  try {
    const pool = await client.fetchPoolState(tokenAMint, tokenBMint);
    console.log("Pool authority:", pool.authority.toBase58());
    console.log("LP fee (bps):", pool.lpFeeBps);
    console.log("Protocol fee (bps):", pool.protocolFeeBps);
    console.log("Paused:", pool.isPaused);
  } catch (error) {
    console.log("Pool not initialized yet or not found:", String(error));
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
