use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

// ============================================================================
// AgentEscrow — Trustless Escrow Protocol for AI Agents on Solana
// ============================================================================
// This program allows AI agents to safely transact with each other.
// Agent A deposits payment into escrow → Agent B does the work →
// Payment is released to Agent B after verification.
// If there's a dispute, verifier agents review and vote.
// ============================================================================

declare_id!("Escrow11111111111111111111111111111111111111");

// ============================================================================
// CONSTANTS
// ============================================================================

const DISPUTE_TIMEOUT_SECONDS: i64 = 86400;        // 24 hours to respond
const VERIFIER_STAKE_MINIMUM: u64 = 1_000_000;      // 1 token minimum stake
const MAX_VERIFIERS_PER_DISPUTE: u8 = 5;            // 5 verifiers per dispute
const ESCROW_TIMEOUT_DEFAULT: i64 = 604800;         // 7 days default timeout

// ============================================================================
// STATES
// ============================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum EscrowState {
    Created,       // Escrow created, waiting for funding
    Funded,        // Payment deposited, waiting for work
    InProgress,    // Agent B started working
    PendingReview, // Work submitted, waiting for Agent A confirmation
    Disputed,      // Dispute opened
    Released,      // Funds released to provider (completed)
    Refunded,      // Funds refunded to payer (completed)
    Expired,       // Timeout reached, auto-handled
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum DisputeState {
    Open,          // Dispute just opened
    Voting,        // Verifiers are voting
    Resolved,      // Voting complete, outcome determined
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum VoteOption {
    FavorPayer,    // Refund the payer
    FavorProvider,  // Pay the provider
    Abstain,        // Verifier abstains
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Main escrow account — stores all info about one transaction
#[account]
pub struct Escrow {
    pub payer: Pubkey,            // Agent A (the one paying)
    pub provider: Pubkey,         // Agent B (the one doing the work)
    pub token_mint: Pubkey,       // What token is being used (USDC, SOL, etc.)
    pub amount: u64,              // How much is being paid
    pub fee_bps: u16,             // Protocol fee in basis points (0.5% = 50)
    pub state: EscrowState,       // Current state of the escrow
    pub created_at: i64,          // When it was created
    pub funded_at: i64,           // When payment was deposited
    pub deadline: i64,            // When the work must be done by
    pub task_hash: [u8; 32],      // Hash of the task description
    pub payer_confirmed: bool,    // Has Agent A confirmed the work is done?
    pub provider_submitted: bool, // Has Agent B submitted the work?
    pub bump: u8,                 // PDA bump seed
}

impl Escrow {
    const LEN: usize = 8 +        // discriminator
        32 +                      // payer
        32 +                      // provider
        32 +                      // token_mint
        8 +                       // amount
        2 +                       // fee_bps
        1 +                       // state
        8 +                       // created_at
        8 +                       // funded_at
        8 +                       // deadline
        32 +                      // task_hash
        1 +                       // payer_confirmed
        1 +                       // provider_submitted
        1;                        // bump
}

/// Dispute account — created when a dispute is opened
#[account]
pub struct Dispute {
    pub escrow: Pubkey,           // Which escrow this dispute is about
    pub opened_by: Pubkey,        // Who opened the dispute
    pub reason_hash: [u8; 32],    // Hash of the reason/evidence
    pub state: DisputeState,      // Current state of the dispute
    pub opened_at: i64,           // When dispute was opened
    pub voting_deadline: i64,     // When voting ends
    pub votes_favor_payer: u8,    // Vote count
    pub votes_favor_provider: u8, // Vote count
    pub votes_abstain: u8,        // Vote count
    pub verifier_count: u8,       // How many verifiers voted
    pub bump: u8,                 // PDA bump seed
}

impl Dispute {
    const LEN: usize = 8 +        // discriminator
        32 +                      // escrow
        32 +                      // opened_by
        32 +                      // reason_hash
        1 +                       // state
        8 +                       // opened_at
        8 +                       // voting_deadline
        1 +                       // votes_favor_payer
        1 +                       // votes_favor_provider
        1 +                       // votes_abstain
        1 +                       // verifier_count
        1;                        // bump
}

/// Verifier registry — agents who stake tokens to become dispute resolvers
#[account]
pub struct Verifier {
    pub agent: Pubkey,            // The verifier agent's address
    pub stake_amount: u64,        // How much they've staked
    pub total_disputes: u32,      // How many disputes they've resolved
    pub correct_votes: u32,       // How many times they voted with the majority
    pub slashed_count: u32,       // How many times they were slashed
    pub is_active: bool,          // Are they currently active?
    pub registered_at: i64,       // When they registered
    pub bump: u8,                 // PDA bump seed
}

impl Verifier {
    const LEN: usize = 8 +        // discriminator
        32 +                      // agent
        8 +                       // stake_amount
        4 +                       // total_disputes
        4 +                       // correct_votes
        4 +                       // slashed_count
        1 +                       // is_active
        8 +                       // registered_at
        1;                        // bump
}

/// Vote record — one verifier's vote on one dispute
#[account]
pub struct VoteRecord {
    pub dispute: Pubkey,          // Which dispute
    pub verifier: Pubkey,         // Who voted
    pub vote: VoteOption,         // What they voted
    pub voted_at: i64,            // When they voted
    pub bump: u8,                 // PDA bump seed
}

impl VoteRecord {
    const LEN: usize = 8 +        // discriminator
        32 +                      // dispute
        32 +                      // verifier
        1 +                       // vote
        8 +                       // voted_at
        1;                        // bump
}

// ============================================================================
// INSTRUCTIONS (Functions that can be called)
// ============================================================================

#[program]
pub mod agentescrow {
    use super::*;

    // --- ESCROW LIFECYCLE ---

    /// Create a new escrow (Agent A calls this)
    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        provider: Pubkey,
        token_mint: Pubkey,
        amount: u64,
        deadline: i64,
        task_hash: [u8; 32],
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        escrow.payer = ctx.accounts.payer.key();
        escrow.provider = provider;
        escrow.token_mint = token_mint;
        escrow.amount = amount;
        escrow.fee_bps = 50; // 0.5% default fee
        escrow.state = EscrowState::Created;
        escrow.created_at = Clock::get()?.unix_timestamp;
        escrow.funded_at = 0;
        escrow.deadline = deadline;
        escrow.task_hash = task_hash;
        escrow.payer_confirmed = false;
        escrow.provider_submitted = false;
        escrow.bump = ctx.bumps.escrow;

        emit!(EscrowCreated {
            escrow: escrow.key(),
            payer: escrow.payer,
            provider: escrow.provider,
            amount: escrow.amount,
            deadline: escrow.deadline,
        });

        Ok(())
    }

    /// Fund the escrow (Agent A deposits the payment)
    pub fn fund_escrow(ctx: Context<FundEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(escrow.state == EscrowState::Created, EscrowError::InvalidState);

        let transfer_amount = escrow.amount;
        let cpi_accounts = Transfer {
            from: ctx.accounts.payer_token_account.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, transfer_amount)?;

        escrow.state = EscrowState::Funded;
        escrow.funded_at = Clock::get()?.unix_timestamp;

        emit!(EscrowFunded {
            escrow: escrow.key(),
            amount: escrow.amount,
        });

        Ok(())
    }

    /// Provider signals they're starting work (Agent B calls this)
    pub fn start_work(ctx: Context<StartWork>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            ctx.accounts.provider.key() == escrow.provider,
            EscrowError::Unauthorized
        );
        require!(escrow.state == EscrowState::Funded, EscrowError::InvalidState);

        escrow.state = EscrowState::InProgress;

        emit!(WorkStarted {
            escrow: escrow.key(),
            provider: ctx.accounts.provider.key(),
        });

        Ok(())
    }

    /// Provider submits work as complete (Agent B calls this)
    pub fn submit_work(ctx: Context<SubmitWork>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            ctx.accounts.provider.key() == escrow.provider,
            EscrowError::Unauthorized
        );
        require!(
            escrow.state == EscrowState::InProgress || escrow.state == EscrowState::Funded,
            EscrowError::InvalidState
        );

        escrow.provider_submitted = true;
        escrow.state = EscrowState::PendingReview;

        emit!(WorkSubmitted {
            escrow: escrow.key(),
            provider: ctx.accounts.provider.key(),
        });

        Ok(())
    }

    /// Payer confirms the work is done and funds release (Agent A calls this)
    pub fn confirm_completion(ctx: Context<ConfirmCompletion>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            ctx.accounts.payer.key() == escrow.payer,
            EscrowError::Unauthorized
        );
        require!(
            escrow.state == EscrowState::PendingReview,
            EscrowError::InvalidState
        );

        let escrow_key = escrow.key();

        release_funds_internal(
            &ctx.accounts.vault.to_account_info(),
            &ctx.accounts.provider_token_account.to_account_info(),
            &ctx.accounts.treasury_token_account.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            escrow,
            &escrow_key,
            ctx.bumps.vault,
        )?;

        escrow.state = EscrowState::Released;

        emit!(EscrowReleased {
            escrow: escrow_key,
            provider: escrow.provider,
            amount: escrow.amount,
        });

        Ok(())
    }

    /// Payer or provider can trigger timeout if the other party is unresponsive
    pub fn trigger_timeout(ctx: Context<TriggerTimeout>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let now = Clock::get()?.unix_timestamp;

        require!(now > escrow.deadline, EscrowError::NotExpiredYet);

        match escrow.state {
            EscrowState::PendingReview => {
                // Payer didn't confirm in time → auto-release to provider
                let escrow_key = escrow.key();
                release_funds_internal(
                    &ctx.accounts.vault.to_account_info(),
                    &ctx.accounts.provider_token_account.to_account_info(),
                    &ctx.accounts.treasury_token_account.to_account_info(),
                    &ctx.accounts.token_program.to_account_info(),
                    escrow,
                    &escrow_key,
                    ctx.bumps.vault,
                )?;
                escrow.state = EscrowState::Expired;
                emit!(EscrowExpired {
                    escrow: escrow_key,
                    reason: "Payer did not confirm, auto-released to provider".to_string(),
                });
            }
            EscrowState::Funded | EscrowState::InProgress => {
                // Provider didn't deliver in time → auto-refund to payer
                let escrow_key = escrow.key();
                refund_funds_internal(
                    &ctx.accounts.vault.to_account_info(),
                    &ctx.accounts.payer_token_account.to_account_info(),
                    &ctx.accounts.token_program.to_account_info(),
                    escrow,
                    &escrow_key,
                    ctx.bumps.vault,
                )?;
                escrow.state = EscrowState::Expired;
                emit!(EscrowExpired {
                    escrow: escrow_key,
                    reason: "Provider did not deliver, auto-refunded to payer".to_string(),
                });
            }
            _ => return Err(EscrowError::InvalidState.into()),
        }

        Ok(())
    }

    // --- DISPUTE RESOLUTION ---

    /// Either party can open a dispute
    pub fn open_dispute(
        ctx: Context<OpenDispute>,
        reason_hash: [u8; 32],
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let dispute = &mut ctx.accounts.dispute;

        require!(
            escrow.state == EscrowState::PendingReview,
            EscrowError::InvalidState
        );

        let opener = ctx.accounts.opener.key();
        require!(
            opener == escrow.payer || opener == escrow.provider,
            EscrowError::Unauthorized
        );

        escrow.state = EscrowState::Disputed;

        dispute.escrow = escrow.key();
        dispute.opened_by = opener;
        dispute.reason_hash = reason_hash;
        dispute.state = DisputeState::Voting;
        dispute.opened_at = Clock::get()?.unix_timestamp;
        dispute.voting_deadline = Clock::get()?.unix_timestamp + DISPUTE_TIMEOUT_SECONDS;
        dispute.votes_favor_payer = 0;
        dispute.votes_favor_provider = 0;
        dispute.votes_abstain = 0;
        dispute.verifier_count = 0;
        dispute.bump = ctx.bumps.dispute;

        emit!(DisputeOpened {
            escrow: escrow.key(),
            dispute: dispute.key(),
            opened_by: opener,
        });

        Ok(())
    }

    /// A registered verifier casts their vote
    pub fn cast_vote(
        ctx: Context<CastVote>,
        vote: VoteOption,
    ) -> Result<()> {
        let dispute = &mut ctx.accounts.dispute;
        let vote_record = &mut ctx.accounts.vote_record;
        let verifier = &mut ctx.accounts.verifier;

        require!(
            dispute.state == DisputeState::Voting,
            EscrowError::VotingClosed
        );

        let now = Clock::get()?.unix_timestamp;
        require!(now < dispute.voting_deadline, EscrowError::VotingClosed);
        require!(verifier.is_active, EscrowError::NotActiveVerifier);

        vote_record.dispute = dispute.key();
        vote_record.verifier = ctx.accounts.verifier_agent.key();
        vote_record.vote = vote;
        vote_record.voted_at = now;
        vote_record.bump = ctx.bumps.vote_record;

        match vote {
            VoteOption::FavorPayer => dispute.votes_favor_payer += 1,
            VoteOption::FavorProvider => dispute.votes_favor_provider += 1,
            VoteOption::Abstain => dispute.votes_abstain += 1,
        }
        dispute.verifier_count += 1;

        emit!(VoteCast {
            dispute: dispute.key(),
            verifier: ctx.accounts.verifier_agent.key(),
            vote,
        });

        if dispute.verifier_count >= MAX_VERIFIERS_PER_DISPUTE {
            dispute.state = DisputeState::Resolved;
            emit!(DisputeResolved {
                escrow: dispute.escrow,
                payer_wins: dispute.votes_favor_payer > dispute.votes_favor_provider,
                votes_payer: dispute.votes_favor_payer,
                votes_provider: dispute.votes_favor_provider,
            });
        }

        Ok(())
    }

    /// Anyone can trigger resolution after voting deadline passes
    pub fn resolve_dispute(ctx: Context<ResolveDispute>) -> Result<()> {
        let dispute = &mut ctx.accounts.dispute;
        let escrow = &mut ctx.accounts.escrow;
        let now = Clock::get()?.unix_timestamp;

        require!(
            dispute.state == DisputeState::Voting,
            EscrowError::AlreadyResolved
        );
        require!(now >= dispute.voting_deadline, EscrowError::VotingStillOpen);

        dispute.state = DisputeState::Resolved;

        let payer_wins = dispute.votes_favor_payer > dispute.votes_favor_provider;

        let escrow_key = escrow.key();

        if payer_wins {
            refund_funds_internal(
                &ctx.accounts.vault.to_account_info(),
                &ctx.accounts.payer_token_account.to_account_info(),
                &ctx.accounts.token_program.to_account_info(),
                escrow,
                &escrow_key,
                ctx.bumps.vault,
            )?;
            escrow.state = EscrowState::Refunded;
        } else {
            release_funds_internal(
                &ctx.accounts.vault.to_account_info(),
                &ctx.accounts.provider_token_account.to_account_info(),
                &ctx.accounts.treasury_token_account.to_account_info(),
                &ctx.accounts.token_program.to_account_info(),
                escrow,
                &escrow_key,
                ctx.bumps.vault,
            )?;
            escrow.state = EscrowState::Released;
        }

        emit!(DisputeResolved {
            escrow: escrow_key,
            payer_wins,
            votes_payer: dispute.votes_favor_payer,
            votes_provider: dispute.votes_favor_provider,
        });

        Ok(())
    }

    // --- VERIFIER REGISTRATION ---

    /// An agent registers as a verifier by staking tokens
    pub fn register_verifier(
        ctx: Context<RegisterVerifier>,
        stake_amount: u64,
    ) -> Result<()> {
        let verifier = &mut ctx.accounts.verifier;

        require!(
            stake_amount >= VERIFIER_STAKE_MINIMUM,
            EscrowError::InsufficientStake
        );

        let cpi_accounts = Transfer {
            from: ctx.accounts.verifier_token_account.to_account_info(),
            to: ctx.accounts.verifier_stake_vault.to_account_info(),
            authority: ctx.accounts.verifier_agent.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, stake_amount)?;

        verifier.agent = ctx.accounts.verifier_agent.key();
        verifier.stake_amount = stake_amount;
        verifier.total_disputes = 0;
        verifier.correct_votes = 0;
        verifier.slashed_count = 0;
        verifier.is_active = true;
        verifier.registered_at = Clock::get()?.unix_timestamp;
        verifier.bump = ctx.bumps.verifier;

        emit!(VerifierRegistered {
            agent: ctx.accounts.verifier_agent.key(),
            stake_amount,
        });

        Ok(())
    }

    /// A verifier can withdraw their stake and deregister
    pub fn deregister_verifier(ctx: Context<DeregisterVerifier>) -> Result<()> {
        let verifier = &mut ctx.accounts.verifier;

        require!(verifier.is_active, EscrowError::NotActiveVerifier);

        let agent_key = ctx.accounts.verifier_agent.key();
        let bump = verifier.bump;
        let stake_amount = verifier.stake_amount;

        let seeds = &[
            agent_key.as_ref(),
            b"verifier_stake",
            &[bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.verifier_stake_vault.to_account_info(),
            to: ctx.accounts.verifier_token_account.to_account_info(),
            authority: ctx.accounts.verifier_stake_vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, stake_amount)?;

        verifier.is_active = false;

        emit!(VerifierDeregistered {
            agent: agent_key,
        });

        Ok(())
    }

}

// ============================================================================
// INTERNAL HELPER FUNCTIONS (outside program module)
// ============================================================================

/// Release funds to provider (minus protocol fee)
pub fn release_funds_internal<'info>(
    vault: &AccountInfo<'info>,
    provider_token_account: &AccountInfo<'info>,
    treasury_token_account: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    escrow: &mut Escrow,
    escrow_key: &Pubkey,
    vault_bump: u8,
) -> Result<()> {
    let fee = (escrow.amount as u128 * escrow.fee_bps as u128 / 10000) as u64;
    let provider_amount = escrow.amount.checked_sub(fee).unwrap_or(0);

    let seeds = &[
        escrow_key.as_ref(),
        b"vault",
        &[vault_bump],
    ];
    let signer = &[&seeds[..]];

    // Transfer to provider
    let cpi_accounts = Transfer {
        from: vault.clone(),
        to: provider_token_account.clone(),
        authority: vault.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer);
    token::transfer(cpi_ctx, provider_amount)?;

    // Transfer fee to treasury
    if fee > 0 {
        let cpi_accounts = Transfer {
            from: vault.clone(),
            to: treasury_token_account.clone(),
            authority: vault.clone(),
        };
        let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer);
        token::transfer(cpi_ctx, fee)?;
    }

    Ok(())
}

/// Refund funds to payer
pub fn refund_funds_internal<'info>(
    vault: &AccountInfo<'info>,
    payer_token_account: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    escrow: &mut Escrow,
    escrow_key: &Pubkey,
    vault_bump: u8,
) -> Result<()> {
    let seeds = &[
        escrow_key.as_ref(),
        b"vault",
        &[vault_bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: vault.clone(),
        to: payer_token_account.clone(),
        authority: vault.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer);
    token::transfer(cpi_ctx, escrow.amount)?;

    Ok(())
}

// ============================================================================
// ACCOUNT VALIDATION CONTEXTS
// ============================================================================

#[derive(Accounts)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = Escrow::LEN + 8,
        seeds = [payer.key().as_ref(), b"escrow"],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct FundEscrow<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, has_one = payer)]
    pub escrow: Account<'info, Escrow>,

    #[account(mut)]
    pub payer_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        token::mint = token_mint,
        token::authority = vault,
        seeds = [escrow.key().as_ref(), b"vault"],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_mint: Account<'info, token::Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct StartWork<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
}

#[derive(Accounts)]
pub struct SubmitWork<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
}

#[derive(Accounts)]
pub struct ConfirmCompletion<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, has_one = payer)]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        seeds = [escrow.key().as_ref(), b"vault"],
        bump,
        token::authority = vault
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub provider_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TriggerTimeout<'info> {
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        seeds = [escrow.key().as_ref(), b"vault"],
        bump,
        token::authority = vault
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub provider_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct OpenDispute<'info> {
    #[account(mut)]
    pub opener: Signer<'info>,

    #[account(mut)]
    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = opener,
        space = Dispute::LEN + 8,
        seeds = [escrow.key().as_ref(), b"dispute"],
        bump,
    )]
    pub dispute: Account<'info, Dispute>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub verifier_agent: Signer<'info>,

    #[account(
        mut,
        seeds = [verifier_agent.key().as_ref(), b"verifier"],
        bump,
    )]
    pub verifier: Account<'info, Verifier>,

    #[account(mut)]
    pub dispute: Account<'info, Dispute>,

    #[account(
        init,
        payer = verifier_agent,
        space = VoteRecord::LEN + 8,
        seeds = [dispute.key().as_ref(), verifier_agent.key().as_ref(), b"vote"],
        bump,
    )]
    pub vote_record: Account<'info, VoteRecord>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,

    #[account(mut)]
    pub dispute: Account<'info, Dispute>,

    #[account(
        mut,
        seeds = [escrow.key().as_ref(), b"vault"],
        bump,
        token::authority = vault
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub provider_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RegisterVerifier<'info> {
    #[account(mut)]
    pub verifier_agent: Signer<'info>,

    #[account(
        init,
        payer = verifier_agent,
        space = Verifier::LEN + 8,
        seeds = [verifier_agent.key().as_ref(), b"verifier"],
        bump,
    )]
    pub verifier: Account<'info, Verifier>,

    #[account(mut)]
    pub verifier_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = verifier_agent,
        token::mint = token_mint,
        token::authority = verifier_stake_vault,
        seeds = [verifier_agent.key().as_ref(), b"verifier_stake"],
        bump,
    )]
    pub verifier_stake_vault: Account<'info, TokenAccount>,

    pub token_mint: Account<'info, token::Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DeregisterVerifier<'info> {
    #[account(mut)]
    pub verifier_agent: Signer<'info>,

    #[account(
        mut,
        seeds = [verifier_agent.key().as_ref(), b"verifier"],
        bump,
    )]
    pub verifier: Account<'info, Verifier>,

    #[account(mut)]
    pub verifier_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [verifier_agent.key().as_ref(), b"verifier_stake"],
        bump,
    )]
    pub verifier_stake_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

// ============================================================================
// EVENTS
// ============================================================================

#[event]
pub struct EscrowCreated {
    pub escrow: Pubkey,
    pub payer: Pubkey,
    pub provider: Pubkey,
    pub amount: u64,
    pub deadline: i64,
}

#[event]
pub struct EscrowFunded {
    pub escrow: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WorkStarted {
    pub escrow: Pubkey,
    pub provider: Pubkey,
}

#[event]
pub struct WorkSubmitted {
    pub escrow: Pubkey,
    pub provider: Pubkey,
}

#[event]
pub struct EscrowReleased {
    pub escrow: Pubkey,
    pub provider: Pubkey,
    pub amount: u64,
}

#[event]
pub struct EscrowExpired {
    pub escrow: Pubkey,
    pub reason: String,
}

#[event]
pub struct DisputeOpened {
    pub escrow: Pubkey,
    pub dispute: Pubkey,
    pub opened_by: Pubkey,
}

#[event]
pub struct VoteCast {
    pub dispute: Pubkey,
    pub verifier: Pubkey,
    pub vote: VoteOption,
}

#[event]
pub struct DisputeResolved {
    pub escrow: Pubkey,
    pub payer_wins: bool,
    pub votes_payer: u8,
    pub votes_provider: u8,
}

#[event]
pub struct VerifierRegistered {
    pub agent: Pubkey,
    pub stake_amount: u64,
}

#[event]
pub struct VerifierDeregistered {
    pub agent: Pubkey,
}

// ============================================================================
// ERRORS
// ============================================================================

#[error_code]
pub enum EscrowError {
    #[msg("The escrow is not in the correct state for this action")]
    InvalidState,
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("The escrow has not expired yet")]
    NotExpiredYet,
    #[msg("Voting has closed for this dispute")]
    VotingClosed,
    #[msg("Voting is still open, wait for the deadline")]
    VotingStillOpen,
    #[msg("This dispute has already been resolved")]
    AlreadyResolved,
    #[msg("Stake amount is below the minimum required")]
    InsufficientStake,
    #[msg("This verifier is not active")]
    NotActiveVerifier,
}
