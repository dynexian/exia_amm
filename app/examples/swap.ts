import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import { ExiaAmmClient, keypairFromFile } from "../src/index";

async function main() {
    const providerUrl = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
    const walletPath = process.env.ANCHOR_WALLET ?? `${process.env.HOME}/.config/solana/id.json`;

    const mintAStr = process.env.TOKEN_A_MINT;
    const mintBStr = process.env.TOKEN_B_MINT;
    const amountInStr = process.env.AMOUNT_IN ?? "10";
    const minOutStr = process.env.MIN_OUT ?? "1";
    const direction = process.env.DIRECTION ?? "AtoB"; // "AtoB" or "BtoA"

    if (!mintAStr || !mintBStr) {
        throw new Error("TOKEN_A_MINT and TOKEN_B_MINT must be set.");
    }

    const mintA = new PublicKey(mintAStr);
    const mintB = new PublicKey(mintBStr);
    const aToB = direction === "AtoB";
    const keypair = await keypairFromFile(walletPath);

    const connection = new anchor.web3.Connection(providerUrl, "confirmed");
    const wallet = new anchor.Wallet(keypair);
    const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });

    const client = ExiaAmmClient.fromProvider(provider);
    const pdas = client.derivePoolPdas(mintA, mintB);

    console.log("=== Exia AMM — Swap ===");
    console.log(`Wallet:    ${keypair.publicKey.toBase58()}`);
    console.log(`Pool:      ${pdas.poolState.toBase58()}`);
    console.log(`Direction: ${direction}`);

    const DECIMALS = 9;
    const amountIn = BigInt(parseFloat(amountInStr) * 10 ** DECIMALS);
    const minOut = BigInt(parseFloat(minOutStr) * 10 ** DECIMALS);

    console.log(`Amount in: ${amountInStr} tokens`);
    console.log(`Min out:   ${minOutStr} tokens`);

    // Treasury accounts — must be set via env vars
    const treasuryAStr = process.env.TREASURY_A;
    const treasuryBStr = process.env.TREASURY_B;
    if (!treasuryAStr || !treasuryBStr) {
        throw new Error("TREASURY_A and TREASURY_B must be set.");
    }
    const treasuryTokenIn = aToB
        ? new PublicKey(treasuryAStr)
        : new PublicKey(treasuryBStr);

    const userTokenIn = await getAssociatedTokenAddress(
        aToB ? mintA : mintB,
        keypair.publicKey
    );
    const userTokenOut = await getAssociatedTokenAddress(
        aToB ? mintB : mintA,
        keypair.publicKey
    );

    // Check balances before
    const balanceBefore = await connection.getTokenAccountBalance(userTokenOut);
    console.log(`\nOut token balance before: ${balanceBefore.value.uiAmountString}`);

    try {
        const sig = await client.swap({
            user: keypair.publicKey,
            tokenAMint: mintA,
            tokenBMint: mintB,
            userTokenIn,
            userTokenOut,
            treasuryTokenIn,
            amountIn,
            minimumAmountOut: minOut,
            aToB,
        }, [], { commitment: "confirmed" });

        console.log("\n✅ Swap Executed Successfully!");
        console.log(`Signature: ${sig}`);
        console.log(`Explorer:  https://explorer.solana.com/tx/${sig}?cluster=devnet`);

        // Check balances after
        const balanceAfter = await connection.getTokenAccountBalance(userTokenOut);
        console.log(`\nOut token balance after: ${balanceAfter.value.uiAmountString}`);
        console.log(`Tokens received: ${
            parseFloat(balanceAfter.value.uiAmountString ?? "0") -
            parseFloat(balanceBefore.value.uiAmountString ?? "0")
        }`);
    } catch (e) {
        console.error("\n❌ Swap Failed:", e);
        process.exit(1);
    }
}

main();
