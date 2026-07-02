import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { ExiaAmmClient, keypairFromFile } from "../src/index";

async function main() {
    const providerUrl = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
    const walletPath = process.env.ANCHOR_WALLET ?? `${process.env.HOME}/.config/solana/id.json`;
    const mintAStr = process.env.TOKEN_A_MINT!;
    const mintBStr = process.env.TOKEN_B_MINT!;
    const newTreasuryAStr = process.env.NEW_TREASURY_A!;
    const newTreasuryBStr = process.env.NEW_TREASURY_B!;

    if (!newTreasuryAStr || !newTreasuryBStr) {
        throw new Error("NEW_TREASURY_A and NEW_TREASURY_B must be set.");
    }

    const mintA = new PublicKey(mintAStr);
    const mintB = new PublicKey(mintBStr);
    const keypair = await keypairFromFile(walletPath);

    const connection = new anchor.web3.Connection(providerUrl, "confirmed");
    const wallet = new anchor.Wallet(keypair);
    const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
    const client = ExiaAmmClient.fromProvider(provider);

    console.log("=== Exia AMM — Rotate Treasury ===");
    console.log(`New Treasury A: ${newTreasuryAStr}`);
    console.log(`New Treasury B: ${newTreasuryBStr}`);

    const sig = await client.rotateTreasury(
        keypair.publicKey,
        mintA,
        mintB,
        new PublicKey(newTreasuryAStr),
        new PublicKey(newTreasuryBStr),
        [],
        { commitment: "confirmed" }
    );

    console.log(`\n✅ Treasury Rotated!`);
    console.log(`Signature: ${sig}`);
    console.log(`Explorer:  https://explorer.solana.com/tx/${sig}?cluster=devnet`);
}

main();
