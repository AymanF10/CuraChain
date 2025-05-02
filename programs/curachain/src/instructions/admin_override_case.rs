use anchor_lang::prelude::*;
use crate::states::{contexts::*, errors::*, PatientCaseVerificationStatus};

pub fn admin_override_case(ctx: Context<AdminOverrideCase>, case_id: String, is_verified: bool) -> Result<()> {
    let patient_case = &mut ctx.accounts.patient_case;

    // Only allow after 10 days
    let now = Clock::get()?.unix_timestamp;
    require!(now >= patient_case.submission_timestamp + 864_000, CuraChainError::VotingPeriodExpired);

    // Only if not already verified
    require!(!patient_case.is_verified, CuraChainError::CaseAlreadyVerified);

    patient_case.is_verified = is_verified;

    // If verified, emit event and (optionally) create escrow
    let message = if is_verified {
        // Optionally: create escrow here if needed (reuse logic from verify_patient)
        "Admin override: case verified after deadline".to_string()
    } else {
        "Admin override: case rejected after deadline".to_string()
    };
    emit!(PatientCaseVerificationStatus {
        message,
        case_id,
        is_verified,
        timestamp: now,
    });
    Ok(())
} 