import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import {
    TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress,
    createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { ExiaAmm } from "../../target/types/exia_amm";

async function main() {
    process.env.ANCHOR_PROVIDER_URL =
        process.env.ANCHOR_PROVIDER_URL ?? "https://api.devnet.solana.com";

    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.ExiaAmm as Program<ExiaAmm>;
    const wallet = provider.wallet;

    console.log("=== Exia AMM — Pool Initialization ===");
    console.log(`Network:      ${process.env.ANCHOR_PROVIDER_URL}`);
    console.log(`Admin wallet: ${wallet.publicKey.toBase58()}`);

    const mintAStr = process.env.TOKEN_A_MINT;
    const mintBStr = process.env.TOKEN_B_MINT;
    const authorityStr = process.env.AUTHORITY ?? wallet.publicKey.toBase58();

    if (!mintAStr || !mintBStr) {
        throw new Error(
            "TOKEN_A_MINT and TOKEN_B_MINT environment variables must be set.\n" +
            "Example: TOKEN_A_MINT=<pubkey> TOKEN_B_MINT=<pubkey> npx ts-node examples/init.ts"
        );
    }

    const mintA = new PublicKey(mintAStr);
    const mintB = new PublicKey(mintBStr);
    const authority = new PublicKey(authorityStr);

    // --- Derive all PDAs ---
    const [poolStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("pool"), mintA.toBuffer(), mintB.toBuffer()],
        program.programId
    );
    const [vaultAPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault_a"), poolStatePda.toBuffer()],
        program.programId
    );
    const [vaultBPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault_b"), poolStatePda.toBuffer()],
        program.programId
    );
    const [lpMintPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("lp_mint"), poolStatePda.toBuffer()],
        program.programId
    );

    console.log("\n--- Derived Addresses ---");
    console.log(`Pool State:  ${poolStatePda.toBase58()}`);
    console.log(`Vault A:     ${vaultAPda.toBase58()}`);
    console.log(`Vault B:     ${vaultBPda.toBase58()}`);
    console.log(`LP Mint:     ${lpMintPda.toBase58()}`);

    // --- Treasury ATAs (admin's token accounts receive protocol fees) ---
    const treasuryTokenA = await getAssociatedTokenAddress(mintA, wallet.publicKey);
    const treasuryTokenB = await getAssociatedTokenAddress(mintB, wallet.publicKey);

    console.log(`Treasury A:  ${treasuryTokenA.toBase58()}`);
    console.log(`Treasury B:  ${treasuryTokenB.toBase58()}`);

    const tx = new anchor.web3.Transaction();

    // Create treasury ATAs if they don't exist
    const [ataAInfo, ataBInfo] = await Promise.all([
        provider.connection.getAccountInfo(treasuryTokenA),
        provider.connection.getAccountInfo(treasuryTokenB),
    ]);

    if (!ataAInfo) {
        console.log("\nCreating Treasury A ATA...");
        tx.add(createAssociatedTokenAccountInstruction(
            wallet.publicKey, treasuryTokenA, wallet.publicKey, mintA
        ));
    }
    if (!ataBInfo) {
        console.log("Creating Treasury B ATA...");
        tx.add(createAssociatedTokenAccountInstruction(
            wallet.publicKey, treasuryTokenB, wallet.publicKey, mintB
        ));
    }

    // --- Build initialize_pool instruction ---
    const initIx = await program.methods
        .initializePool(
            25,          // lp_fee_bps  — 0.25%
            5,           // protocol_fee_bps — 0.05%
            authority    // admin authority pubkey
        )
        .accounts({
            payer:          wallet.publicKey,
            poolState:      poolStatePda,
            tokenAMint:     mintA,
            tokenBMint:     mintB,
            treasuryTokenA: treasuryTokenA,
            treasuryTokenB: treasuryTokenB,
            vaultA:         vaultAPda,
            vaultB:         vaultBPda,
            lpMint:         lpMintPda,
            systemProgram:  SystemProgram.programId,
            tokenProgram:   TOKEN_PROGRAM_ID,
            rent:           SYSVAR_RENT_PUBKEY,
        } as any)
        .instruction();

    tx.add(initIx);

    try {
        console.log("\nSending transaction...");
        const sig = await provider.sendAndConfirm(tx, [], { commitment: "confirmed" });
        console.log("\n✅ Pool Initialized Successfully!");
        console.log(`Signature: ${sig}`);
        console.log(`Explorer:  https://explorer.solana.com/tx/${sig}?cluster=devnet`);
        console.log("\n--- Pool Summary ---");
        console.log(`Pool State PDA: ${poolStatePda.toBase58()}`);
        console.log(`LP Mint:        ${lpMintPda.toBase58()}`);
        console.log(`LP Fee:         0.25% (25 bps)`);
        console.log(`Protocol Fee:   0.05% (5 bps)`);
        console.log(`Authority:      ${authority.toBase58()}`);
    } catch (e) {
        console.error("\n❌ Initialization Failed:", e);
        process.exit(1);
    }
}

main();
