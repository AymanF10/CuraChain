use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Verifier not whitelisted")]
    VerifierNotWhitelisted,
    #[msg("Verification timeout")]
    VerificationTimeout,
    #[msg("Case not approved")]
    CaseNotApproved,
    #[msg("Invalid donation amount")]
    InvalidDonationAmount,
    #[msg("Insufficient participation")]
    InsufficientParticipation,
    #[msg("Duplicate vote")]
    DuplicateVote,
}