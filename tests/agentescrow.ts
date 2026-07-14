import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Agentescrow } from "../target/types/agentescrow";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { expect } from "chai";

describe("AgentEscrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Agentescrow as Program<Agentescrow>;
  const payer = provider.wallet as anchor.Wallet;

  // Test token mint and accounts
  let tokenMint: PublicKey;
  let payerTokenAccount: PublicKey;
  let providerKeypair: Keypair;
  let providerTokenAccount: PublicKey;
  let treasuryTokenAccount: PublicKey;

  before(async () => {
    // Create a test token mint
    tokenMint = await createMint(
      provider.connection,
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      6
    );

    // Create token accounts
    payerTokenAccount = await createAccount(
      provider.connection,
      payer.payer,
      tokenMint,
      payer.publicKey
    );

    providerKeypair = Keypair.generate();
    providerTokenAccount = await createAccount(
      provider.connection,
      payer.payer,
      tokenMint,
      providerKeypair.publicKey
    );

    treasuryTokenAccount = await createAccount(
      provider.connection,
      payer.payer,
      tokenMint,
      payer.publicKey
    );

    // Mint tokens to payer
    await mintTo(
      provider.connection,
      payer.payer,
      tokenMint,
      payerTokenAccount,
      payer.payer,
      1_000_000_000 // 1000 tokens
    );
  });

  it("Creates an escrow", async () => {
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [payer.publicKey.toBytes(), Buffer.from("escrow")],
      program.programId
    );

    const taskHash = Buffer.alloc(32);
    taskHash.write("test task description");
    const deadline = Math.floor(Date.now() / 1000) + 604800; // 7 days

    await program.methods
      .createEscrow(
        providerKeypair.publicKey,
        tokenMint,
        new anchor.BN(100_000_000), // 100 tokens
        new anchor.BN(deadline),
        taskHash
      )
      .accounts({
        payer: payer.publicKey,
        escrow: escrowPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    expect(escrowAccount.payer.toBase58()).to.equal(payer.publicKey.toBase58());
    expect(escrowAccount.provider.toBase58()).to.equal(providerKeypair.publicKey.toBase58());
    expect(escrowAccount.amount.toNumber()).to.equal(100_000_000);
    expect(escrowAccount.state).to.deep.equal({ created: {} });
  });

  it("Funds the escrow", async () => {
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [payer.publicKey.toBytes(), Buffer.from("escrow")],
      program.programId
    );

    const [vaultPda] = PublicKey.findProgramAddressSync(
      [escrowPda.toBytes(), Buffer.from("vault")],
      program.programId
    );

    await program.methods
      .fundEscrow()
      .accounts({
        payer: payer.publicKey,
        escrow: escrowPda,
        payerTokenAccount,
        vault: vaultPda,
        tokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    expect(escrowAccount.state).to.deep.equal({ funded: {} });

    const vaultAccount = await getAccount(provider.connection, vaultPda);
    expect(Number(vaultAccount.amount)).to.equal(100_000_000);
  });

  it("Provider starts work", async () => {
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [payer.publicKey.toBytes(), Buffer.from("escrow")],
      program.programId
    );

    await program.methods
      .startWork()
      .accounts({
        provider: providerKeypair.publicKey,
        escrow: escrowPda,
      })
      .signers([providerKeypair])
      .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    expect(escrowAccount.state).to.deep.equal({ inProgress: {} });
  });

  it("Provider submits work", async () => {
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [payer.publicKey.toBytes(), Buffer.from("escrow")],
      program.programId
    );

    await program.methods
      .submitWork()
      .accounts({
        provider: providerKeypair.publicKey,
        escrow: escrowPda,
      })
      .signers([providerKeypair])
      .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    expect(escrowAccount.state).to.deep.equal({ pendingReview: {} });
    expect(escrowAccount.providerSubmitted).to.be.true;
  });

  it("Payer confirms completion and funds release to provider", async () => {
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [payer.publicKey.toBytes(), Buffer.from("escrow")],
      program.programId
    );

    const [vaultPda] = PublicKey.findProgramAddressSync(
      [escrowPda.toBytes(), Buffer.from("vault")],
      program.programId
    );

    const providerBalanceBefore = await getAccount(provider.connection, providerTokenAccount);

    await program.methods
      .confirmCompletion()
      .accounts({
        payer: payer.publicKey,
        escrow: escrowPda,
        vault: vaultPda,
        providerTokenAccount,
        treasuryTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    expect(escrowAccount.state).to.deep.equal({ released: {} });

    const providerBalanceAfter = await getAccount(provider.connection, providerTokenAccount);
    const diff = Number(providerBalanceAfter.amount) - Number(providerBalanceBefore.amount);
    
    // Provider gets amount minus 0.5% fee
    const expectedAmount = 100_000_000 - 500_000; // 99.5 tokens
    expect(diff).to.equal(expectedAmount);
  });
});
