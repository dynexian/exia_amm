import { AnchorProvider, Wallet } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { ExiaAmmClient, keypairFromFile } from "../src/index.js";

const REQUIRED_CONFIRMATION = "YES_I_KNOW_THIS_SENDS_A_TX";

async function main() {
  const rpcUrl = process.env.RPC_URL ?? "http://127.0.0.1:8899";
  const walletPath = process.env.WALLET_PATH ?? `${process.env.HOME}/.config/solana/id.json`;
  const tokenAMintRaw = process.env.TOKEN_A_MINT;
  const tokenBMintRaw = process.env.TOKEN_B_MINT;
  const confirmation = process.env.WRITE_DEMO_CONFIRM;

  if (!tokenAMintRaw || !tokenBMintRaw) {
    throw new Error("Set TOKEN_A_MINT and TOKEN_B_MINT environment variables.");
  }

  if (confirmation !== REQUIRED_CONFIRMATION) {
    throw new Error(
      `Refusing to send transaction. Set WRITE_DEMO_CONFIRM=${REQUIRED_CONFIRMATION}`,
    );
  }

  const tokenAMint = new PublicKey(tokenAMintRaw);
  const tokenBMint = new PublicKey(tokenBMintRaw);

  const signer = await keypairFromFile(walletPath);
  const wallet = new Wallet(signer);
  const connection = new Connection(rpcUrl, "confirmed");
  const provider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });
  const client = ExiaAmmClient.fromProvider(provider);

  const pool = await client.fetchPoolState(tokenAMint, tokenBMint);

  const lpFeeBps = Number(pool.lpFeeBps);
  const protocolFeeBps = Number(pool.protocolFeeBps);

  console.log("Submitting updateFees with current values (state-preserving smoke write)...");
  console.log("Authority:", wallet.publicKey.toBase58());
  console.log("LP fee bps:", lpFeeBps);
  console.log("Protocol fee bps:", protocolFeeBps);

  const signature = await client.updateFees(
    wallet.publicKey,
    tokenAMint,
    tokenBMint,
    lpFeeBps,
    protocolFeeBps,
  );

  console.log("Transaction signature:", signature);
  console.log("Explorer:", `https://explorer.solana.com/tx/${signature}?cluster=devnet`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
