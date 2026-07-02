import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import { ExiaAmmClient, keypairFromFile } from "../src/index";

async function main() {
    const providerUrl = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
    const walletPath = process.env.ANCHOR_WALLET ?? `${process.env.HOME}/.config/solana/id.json`;

    const mintAStr = process.env.TOKEN_A_MINT;
    const mintBStr = process.env.TOKEN_B_MINT;
    const lpAmountStr = process.env.LP_AMOUNT ?? "10";

    if (!mintAStr || !mintBStr) {
        throw new Error("TOKEN_A_MINT and TOKEN_B_MINT must be set.");
    }

    const mintA = new PublicKey(mintAStr);
    const mintB = new PublicKey(mintBStr);
    const keypair = await keypairFromFile(walletPath);

    const connection = new anchor.web3.Connection(providerUrl, "confirmed");
    const wallet = new anchor.Wallet(keypair);
    const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });

    const client = ExiaAmmClient.fromProvider(provider);
    const pdas = client.derivePoolPdas(mintA, mintB);

    console.log("=== Exia AMM — Remove Liquidity ===");
    console.log(`Wallet:     ${keypair.publicKey.toBase58()}`);
    console.log(`Pool:       ${pdas.poolState.toBase58()}`);
    console.log(`LP Amount:  ${lpAmountStr} LP tokens`);

    const DECIMALS = 9;
    const lpAmount = BigInt(parseFloat(lpAmountStr) * 10 ** DECIMALS);

    const userTokenA = await getAssociatedTokenAddress(mintA, keypair.publicKey);
    const userTokenB = await getAssociatedTokenAddress(mintB, keypair.publicKey);
    const userLpToken = await getAssociatedTokenAddress(pdas.lpMint, keypair.publicKey);

    // Balances before
    const [balABefore, balBBefore, balLpBefore] = await Promise.all([
        connection.getTokenAccountBalance(userTokenA),
        connection.getTokenAccountBalance(userTokenB),
        connection.getTokenAccountBalance(userLpToken),
    ]);

    console.log(`\nBalances before:`);
    console.log(`  Token A: ${balABefore.value.uiAmountString}`);
    console.log(`  Token B: ${balBBefore.value.uiAmountString}`);
    console.log(`  LP:      ${balLpBefore.value.uiAmountString}`);

    try {
        const sig = await client.removeLiquidity({
            user: keypair.publicKey,
            tokenAMint: mintA,
            tokenBMint: mintB,
            userTokenA,
            userTokenB,
            userLpToken,
            lpAmount,
        }, [], { commitment: "confirmed" });

        console.log(`\n✅ Liquidity Removed Successfully!`);
        console.log(`Signature: ${sig}`);
        console.log(`Explorer:  https://explorer.solana.com/tx/${sig}?cluster=devnet`);

        // Balances after
        const [balAAfter, balBAfter, balLpAfter] = await Promise.all([
            connection.getTokenAccountBalance(userTokenA),
            connection.getTokenAccountBalance(userTokenB),
            connection.getTokenAccountBalance(userLpToken),
        ]);

        console.log(`\nBalances after:`);
        console.log(`  Token A: ${balAAfter.value.uiAmountString}`);
        console.log(`  Token B: ${balBAfter.value.uiAmountString}`);
        console.log(`  LP:      ${balLpAfter.value.uiAmountString}`);

        console.log(`\nReturned:`);
        console.log(`  Token A: +${(parseFloat(balAAfter.value.uiAmountString ?? "0") - parseFloat(balABefore.value.uiAmountString ?? "0")).toFixed(9)}`);
        console.log(`  Token B: +${(parseFloat(balBAfter.value.uiAmountString ?? "0") - parseFloat(balBBefore.value.uiAmountString ?? "0")).toFixed(9)}`);
        console.log(`  LP burned: ${lpAmountStr}`);
    } catch (e) {
        console.error("\n❌ Failed:", e);
        process.exit(1);
    }
}

main();
