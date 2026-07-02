import * as anchor from "@coral-xyz/anchor";
import { Program, type Idl } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress,
    createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { ExiaAmmClient, keypairFromFile } from "../src/index";

async function main() {
    const providerUrl = process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";
    const walletPath = process.env.ANCHOR_WALLET ?? `${process.env.HOME}/.config/solana/id.json`;

    const mintAStr = process.env.TOKEN_A_MINT;
    const mintBStr = process.env.TOKEN_B_MINT;
    const amountAStr = process.env.AMOUNT_A ?? "100";
    const amountBStr = process.env.AMOUNT_B ?? "100";

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

    console.log("=== Exia AMM — Add Liquidity ===");
    console.log(`Wallet:    ${keypair.publicKey.toBase58()}`);
    console.log(`Pool:      ${pdas.poolState.toBase58()}`);

    // Token amounts (accounting for 9 decimals)
    const DECIMALS = 9;
    const amountA = BigInt(parseFloat(amountAStr) * 10 ** DECIMALS);
    const amountB = BigInt(parseFloat(amountBStr) * 10 ** DECIMALS);

    console.log(`Amount A:  ${amountAStr} tokens`);
    console.log(`Amount B:  ${amountBStr} tokens`);

    // Get or create user token accounts
    const userTokenA = await getAssociatedTokenAddress(mintA, keypair.publicKey);
    const userTokenB = await getAssociatedTokenAddress(mintB, keypair.publicKey);
    const userLpToken = await getAssociatedTokenAddress(pdas.lpMint, keypair.publicKey);

    const tx = new anchor.web3.Transaction();

    // Create LP token ATA if it doesn't exist
    const lpAtaInfo = await connection.getAccountInfo(userLpToken);
    if (!lpAtaInfo) {
        console.log("Creating LP token account...");
        tx.add(createAssociatedTokenAccountInstruction(
            keypair.publicKey, userLpToken, keypair.publicKey, pdas.lpMint
        ));
    }

    // Build add_liquidity instruction via client
    const addLiqIx = await client.program.methods
        .addLiquidity(
            new anchor.BN(amountA.toString()),
            new anchor.BN(amountB.toString())
        )
        .accounts({
            user: keypair.publicKey,
            poolState: pdas.poolState,
            userTokenA,
            userTokenB,
            userLpToken,
            vaultA: pdas.vaultA,
            vaultB: pdas.vaultB,
            lpMint: pdas.lpMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
        } as any)
        .instruction();

    tx.add(addLiqIx);

    try {
        const sig = await provider.sendAndConfirm(tx, [], { commitment: "confirmed" });
        console.log("\n✅ Liquidity Added Successfully!");
        console.log(`Signature: ${sig}`);
        console.log(`Explorer:  https://explorer.solana.com/tx/${sig}?cluster=devnet`);

        // Show LP balance
        const lpBalance = await connection.getTokenAccountBalance(userLpToken);
        console.log(`\nLP tokens received: ${lpBalance.value.uiAmountString}`);
    } catch (e) {
        console.error("\n❌ Failed:", e);
        process.exit(1);
    }
}

main();
