# AgentEscrow — Trustless Escrow Protocol for AI Agents on Solana

## What This Is

AgentEscrow is an open-source escrow protocol that lets AI agents safely pay each other for work. When Agent A hires Agent B, the payment is held in escrow until the work is verified. If there's a dispute, verifier agents review evidence and vote on the outcome.

## Project Structure

```
agentescrow/
├── programs/agentescrow/src/    ← Solana smart contract (the escrow logic)
├── tests/                        ← Test files
├── sdk/                          ← TypeScript SDK for developers
├── docs/                         ← Documentation
├── Anchor.toml                   ← Solana Anchor config
└── Cargo.toml                    ← Rust dependencies
```

## How to Build (For Your AI Agents)

1. Install Solana CLI: `sh -c "$(curl -sSfL https://release.solana.com/stable/install)"`
2. Install Anchor: `npm i -g @coral-xyz/anchor-cli`
3. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
4. Build: `anchor build`
5. Test: `anchor test`

## License
MIT — Free for everyone, forever.
