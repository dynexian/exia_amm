import { AnchorProvider, BN, Program, type Idl, type Wallet } from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  type Connection,
  type SendOptions,
  type Signer,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import idlJson from "../../target/idl/exia_amm.json" with { type: "json" };

export const EXIA_AMM_PROGRAM_ID = new PublicKey(idlJson.address);

export type Numeric = number | bigint | BN;

export type PoolPdas = {
  poolState: PublicKey;
  vaultA: PublicKey;
  vaultB: PublicKey;
  lpMint: PublicKey;
};

export type InitPoolParams = {
  payer: PublicKey;
  tokenAMint: PublicKey;
  tokenBMint: PublicKey;
  treasuryTokenA: PublicKey;
  treasuryTokenB: PublicKey;
  authority: PublicKey;
  lpFeeBps: number;
  protocolFeeBps: number;
};

export type AddLiquidityParams = {
  user: PublicKey;
  tokenAMint: PublicKey;
  tokenBMint: PublicKey;
  userTokenA: PublicKey;
  userTokenB: PublicKey;
  userLpToken: PublicKey;
  amountA: Numeric;
  amountB: Numeric;
};

export type SwapParams = {
  user: PublicKey;
  tokenAMint: PublicKey;
  tokenBMint: PublicKey;
  userTokenIn: PublicKey;
  userTokenOut: PublicKey;
  treasuryTokenIn: PublicKey;
  amountIn: Numeric;
  minimumAmountOut: Numeric;
  aToB: boolean;
};

export type RemoveLiquidityParams = {
  user: PublicKey;
  tokenAMint: PublicKey;
  tokenBMint: PublicKey;
  userTokenA: PublicKey;
  userTokenB: PublicKey;
  userLpToken: PublicKey;
  lpAmount: Numeric;
};

function toBn(value: Numeric): BN {
  if (BN.isBN(value)) {
    return value;
  }
  return new BN(value.toString());
}

export function derivePoolPdas(
  tokenAMint: PublicKey,
  tokenBMint: PublicKey,
  programId: PublicKey = EXIA_AMM_PROGRAM_ID,
): PoolPdas {
  const [poolState] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), tokenAMint.toBuffer(), tokenBMint.toBuffer()],
    programId,
  );

  const [vaultA] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_a"), poolState.toBuffer()],
    programId,
  );

  const [vaultB] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_b"), poolState.toBuffer()],
    programId,
  );

  const [lpMint] = PublicKey.findProgramAddressSync(
    [Buffer.from("lp_mint"), poolState.toBuffer()],
    programId,
  );

  return { poolState, vaultA, vaultB, lpMint };
}

export function getProgram(
  provider: AnchorProvider,
  programId: PublicKey = EXIA_AMM_PROGRAM_ID,
): Program<Idl> {
  const idl = {
    ...(idlJson as object),
    address: programId.toBase58(),
  } as Idl;
  return new Program(idl, provider);
}

export class ExiaAmmClient {
  readonly program: Program<Idl>;

  constructor(program: Program<Idl>) {
    this.program = program;
  }

  static fromProvider(
    provider: AnchorProvider,
    programId: PublicKey = EXIA_AMM_PROGRAM_ID,
  ): ExiaAmmClient {
    return new ExiaAmmClient(getProgram(provider, programId));
  }

  static fromConnection(
    connection: Connection,
    wallet: Wallet,
    opts: ConstructorParameters<typeof AnchorProvider>[2],
    programId: PublicKey = EXIA_AMM_PROGRAM_ID,
  ): ExiaAmmClient {
    const provider = new AnchorProvider(connection, wallet, opts);
    return ExiaAmmClient.fromProvider(provider, programId);
  }

  derivePoolPdas(tokenAMint: PublicKey, tokenBMint: PublicKey): PoolPdas {
    return derivePoolPdas(tokenAMint, tokenBMint, this.program.programId);
  }

  async fetchPoolState(tokenAMint: PublicKey, tokenBMint: PublicKey) {
    const { poolState } = this.derivePoolPdas(tokenAMint, tokenBMint);
    return (this.program.account as any).poolState.fetch(poolState);
  }

  async initializePool(
    params: InitPoolParams,
    signers: Signer[] = [],
    options?: SendOptions,
  ): Promise<string> {
    const pdas = this.derivePoolPdas(params.tokenAMint, params.tokenBMint);

    return this.program.methods
      .initializePool(params.lpFeeBps, params.protocolFeeBps, params.authority)
      .accounts({
        payer: params.payer,
        poolState: pdas.poolState,
        tokenAMint: params.tokenAMint,
        tokenBMint: params.tokenBMint,
        treasuryTokenA: params.treasuryTokenA,
        treasuryTokenB: params.treasuryTokenB,
        vaultA: pdas.vaultA,
        vaultB: pdas.vaultB,
        lpMint: pdas.lpMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      } as any)
      .signers(signers)
      .rpc(options);
  }

  async addLiquidity(
    params: AddLiquidityParams,
    signers: Signer[] = [],
    options?: SendOptions,
  ): Promise<string> {
    const pdas = this.derivePoolPdas(params.tokenAMint, params.tokenBMint);

    return this.program.methods
      .addLiquidity(toBn(params.amountA), toBn(params.amountB))
      .accounts({
        user: params.user,
        poolState: pdas.poolState,
        userTokenA: params.userTokenA,
        userTokenB: params.userTokenB,
        userLpToken: params.userLpToken,
        vaultA: pdas.vaultA,
        vaultB: pdas.vaultB,
        lpMint: pdas.lpMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers(signers)
      .rpc(options);
  }

  async swap(
    params: SwapParams,
    signers: Signer[] = [],
    options?: SendOptions,
  ): Promise<string> {
    const pdas = this.derivePoolPdas(params.tokenAMint, params.tokenBMint);

    return this.program.methods
      .swap(toBn(params.amountIn), toBn(params.minimumAmountOut), params.aToB)
      .accounts({
        user: params.user,
        poolState: pdas.poolState,
        userTokenIn: params.userTokenIn,
        userTokenOut: params.userTokenOut,
        vaultA: pdas.vaultA,
        vaultB: pdas.vaultB,
        treasuryTokenIn: params.treasuryTokenIn,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers(signers)
      .rpc(options);
  }

  async removeLiquidity(
    params: RemoveLiquidityParams,
    signers: Signer[] = [],
    options?: SendOptions,
  ): Promise<string> {
    const pdas = this.derivePoolPdas(params.tokenAMint, params.tokenBMint);

    return this.program.methods
      .removeLiquidity(toBn(params.lpAmount))
      .accounts({
        user: params.user,
        poolState: pdas.poolState,
        userTokenA: params.userTokenA,
        userTokenB: params.userTokenB,
        userLpToken: params.userLpToken,
        vaultA: pdas.vaultA,
        vaultB: pdas.vaultB,
        lpMint: pdas.lpMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers(signers)
      .rpc(options);
  }

  async updateFees(
    authority: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    newLpFeeBps: number,
    newProtocolFeeBps: number,
    signers: Signer[] = [],
    options?: SendOptions,
  ): Promise<string> {
    const { poolState } = this.derivePoolPdas(tokenAMint, tokenBMint);

    return this.program.methods
      .updateFees(newLpFeeBps, newProtocolFeeBps)
      .accounts({ authority, poolState } as any)
      .signers(signers)
      .rpc(options);
  }

  async setPaused(
    authority: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    paused: boolean,
    signers: Signer[] = [],
    options?: SendOptions,
  ): Promise<string> {
    const { poolState } = this.derivePoolPdas(tokenAMint, tokenBMint);

    return this.program.methods
      .setPaused(paused)
      .accounts({ authority, poolState } as any)
      .signers(signers)
      .rpc(options);
  }

  async rotateTreasury(
    authority: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    newTreasuryTokenA: PublicKey,
    newTreasuryTokenB: PublicKey,
    signers: Signer[] = [],
    options?: SendOptions,
  ): Promise<string> {
    const { poolState } = this.derivePoolPdas(tokenAMint, tokenBMint);

    return this.program.methods
      .rotateTreasury()
      .accounts({
        authority,
        poolState,
        newTreasuryTokenA,
        newTreasuryTokenB,
      } as any)
      .signers(signers)
      .rpc(options);
  }

  async proposeAuthority(
    authority: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    newAuthority: PublicKey,
    signers: Signer[] = [],
    options?: SendOptions,
  ): Promise<string> {
    const { poolState } = this.derivePoolPdas(tokenAMint, tokenBMint);

    return this.program.methods
      .proposeAuthority(newAuthority)
      .accounts({ authority, poolState } as any)
      .signers(signers)
      .rpc(options);
  }

  async acceptAuthority(
    newAuthority: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    signers: Signer[] = [],
    options?: SendOptions,
  ): Promise<string> {
    const { poolState } = this.derivePoolPdas(tokenAMint, tokenBMint);

    return this.program.methods
      .acceptAuthority()
      .accounts({
        newAuthority,
        poolState,
      } as any)
      .signers(signers)
      .rpc(options);
  }
}

export async function keypairFromFile(path: string): Promise<Keypair> {
  const { readFile } = await import("node:fs/promises");
  const secret = JSON.parse(await readFile(path, "utf8")) as number[];
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}
