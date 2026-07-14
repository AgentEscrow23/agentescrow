/**
 * AgentEscrow SDK — TypeScript SDK for AI agents
 *
 * This SDK lets any AI agent create escrows, fund them,
 * submit work, confirm completion, and handle disputes.
 *
 * Usage:
 *   import { AgentEscrowSDK } from './sdk';
 *   const sdk = new AgentEscrowSDK(provider, programId);
 *   await sdk.createEscrow({ provider, tokenMint, amount, deadline, taskHash });
 */

import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

export class AgentEscrowSDK {
  program: anchor.Program;
  provider: anchor.Provider;

  constructor(provider: anchor.Provider, programId?: PublicKey) {
    this.provider = provider;
    const idl = anchor.workspace.Agentescrow?.idl;
    this.program = new anchor.Program(
      idl as any,
      programId || new PublicKey("Escrow11111111111111111111111111111111111111"),
      provider
    );
  }

  // --- ESCROW LIFECYCLE ---

  /**
   * Create a new escrow
   * @param params.provider - The agent doing the work (Agent B)
   * @param params.tokenMint - The token to use for payment (e.g. USDC)
   * @param params.amount - Amount to pay (in token's smallest unit)
   * @param params.deadline - Unix timestamp when work must be done
   * @param params.taskHash - 32-byte hash of the task description
   */
  async createEscrow(params: {
    provider: PublicKey;
    tokenMint: PublicKey;
    amount: number;
    deadline: number;
    taskHash: Buffer;
  }) {
    const payer = (this.provider.wallet as anchor.Wallet).publicKey;
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [payer.toBytes(), Buffer.from("escrow")],
      this.program.programId
    );

    return this.program.methods
      .createEscrow(
        params.provider,
        params.tokenMint,
        new anchor.BN(params.amount),
        new anchor.BN(params.deadline),
        params.taskHash
      )
      .accounts({
        payer,
        escrow: escrowPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  /**
   * Fund the escrow (deposit payment)
   * @param params.escrow - The escrow account address
   * @param params.payerTokenAccount - Payer's token account
   * @param params.tokenMint - The token mint
   */
  async fundEscrow(params: {
    escrow: PublicKey;
    payerTokenAccount: PublicKey;
    tokenMint: PublicKey;
  }) {
    const payer = (this.provider.wallet as anchor.Wallet).publicKey;
    const [vaultPda] = PublicKey.findProgramAddressSync(
      [params.escrow.toBytes(), Buffer.from("vault")],
      this.program.programId
    );

    return this.program.methods
      .fundEscrow()
      .accounts({
        payer,
        escrow: params.escrow,
        payerTokenAccount: params.payerTokenAccount,
        vault: vaultPda,
        tokenMint: params.tokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  /**
   * Provider starts work on the task
   * @param params.escrow - The escrow account address
   */
  async startWork(params: { escrow: PublicKey }) {
    const provider = (this.provider.wallet as anchor.Wallet).publicKey;
    return this.program.methods
      .startWork()
      .accounts({
        provider,
        escrow: params.escrow,
      })
      .rpc();
  }

  /**
   * Provider submits work as complete
   * @param params.escrow - The escrow account address
   */
  async submitWork(params: { escrow: PublicKey }) {
    const provider = (this.provider.wallet as anchor.Wallet).publicKey;
    return this.program.methods
      .submitWork()
      .accounts({
        provider,
        escrow: params.escrow,
      })
      .rpc();
  }

  /**
   * Payer confirms the work is done — releases funds to provider
   * @param params.escrow - The escrow account address
   * @param params.providerTokenAccount - Provider's token account
   * @param params.treasuryTokenAccount - Treasury token account (for fees)
   */
  async confirmCompletion(params: {
    escrow: PublicKey;
    vault: PublicKey;
    providerTokenAccount: PublicKey;
    treasuryTokenAccount: PublicKey;
  }) {
    const payer = (this.provider.wallet as anchor.Wallet).publicKey;
    return this.program.methods
      .confirmCompletion()
      .accounts({
        payer,
        escrow: params.escrow,
        vault: params.vault,
        providerTokenAccount: params.providerTokenAccount,
        treasuryTokenAccount: params.treasuryTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  }

  // --- DISPUTE RESOLUTION ---

  /**
   * Open a dispute (either payer or provider can call this)
   * @param params.escrow - The escrow account address
   * @param params.reasonHash - 32-byte hash of the dispute reason/evidence
   */
  async openDispute(params: {
    escrow: PublicKey;
    reasonHash: Buffer;
  }) {
    const opener = (this.provider.wallet as anchor.Wallet).publicKey;
    const [disputePda] = PublicKey.findProgramAddressSync(
      [params.escrow.toBytes(), Buffer.from("dispute")],
      this.program.programId
    );

    return this.program.methods
      .openDispute(params.reasonHash)
      .accounts({
        opener,
        escrow: params.escrow,
        dispute: disputePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  /**
   * Cast a vote on a dispute (verifier only)
   * @param params.dispute - The dispute account address
   * @param params.vote - "favorPayer", "favorProvider", or "abstain"
   */
  async castVote(params: {
    dispute: PublicKey;
    vote: "favorPayer" | "favorProvider" | "abstain";
  }) {
    const verifier = (this.provider.wallet as anchor.Wallet).publicKey;
    const [verifierPda] = PublicKey.findProgramAddressSync(
      [verifier.toBytes(), Buffer.from("verifier")],
      this.program.programId
    );
    const [votePda] = PublicKey.findProgramAddressSync(
      [params.dispute.toBytes(), verifier.toBytes(), Buffer.from("vote")],
      this.program.programId
    );

    const voteEnum = { [params.vote]: {} };

    return this.program.methods
      .castVote(voteEnum)
      .accounts({
        verifierAgent: verifier,
        verifier: verifierPda,
        dispute: params.dispute,
        voteRecord: votePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  // --- VERIFIER REGISTRATION ---

  /**
   * Register as a verifier by staking tokens
   * @param params.stakeAmount - Amount to stake
   * @param params.tokenMint - Token to stake
   * @param params.verifierTokenAccount - Your token account
   */
  async registerVerifier(params: {
    stakeAmount: number;
    tokenMint: PublicKey;
    verifierTokenAccount: PublicKey;
  }) {
    const verifier = (this.provider.wallet as anchor.Wallet).publicKey;
    const [verifierPda] = PublicKey.findProgramAddressSync(
      [verifier.toBytes(), Buffer.from("verifier")],
      this.program.programId
    );
    const [stakeVaultPda] = PublicKey.findProgramAddressSync(
      [verifier.toBytes(), Buffer.from("verifier_stake")],
      this.program.programId
    );

    return this.program.methods
      .registerVerifier(new anchor.BN(params.stakeAmount))
      .accounts({
        verifierAgent: verifier,
        verifier: verifierPda,
        verifierTokenAccount: params.verifierTokenAccount,
        verifierStakeVault: stakeVaultPda,
        tokenMint: params.tokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  // --- QUERIES ---

  /**
   * Get escrow details
   */
  async getEscrow(escrowAddress: PublicKey) {
    return this.program.account.escrow.fetch(escrowAddress);
  }

  /**
   * Get dispute details
   */
  async getDispute(disputeAddress: PublicKey) {
    return this.program.account.dispute.fetch(disputeAddress);
  }

  /**
   * Get verifier details
   */
  async getVerifier(verifierAddress: PublicKey) {
    return this.program.account.verifier.fetch(verifierAddress);
  }
}
