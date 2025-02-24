use anchor_lang::prelude::*;
use crate::{
    constants::*,
    state::{PatientCase, CaseStatus},  
    errors::ErrorCode
};

#[derive(Accounts)]
#[instruction(case_id: u64)]
pub struct FinalizeVerification<'info> {
    #[account(mut)]
    pub patient_case: Account<'info, PatientCase>,
}
impl<'info>FinalizeVerification<'info> {
pub fn handler(ctx: Context<FinalizeVerification>, _case_id: u64) -> Result<()> {
    let case = &mut ctx.accounts.patient_case;
    
    // Time check
    require!(
        Clock::get()?.unix_timestamp - case.timestamp > VERIFICATION_PERIOD,
        ErrorCode::VerificationTimeout
    );
    
    // Participation check
    let total_votes = case.approve_votes + case.reject_votes;
    require!(
        (total_votes as f64 / case.total_verifiers as f64) >= MIN_VERIFICATION_PARTICIPATION,
        ErrorCode::InsufficientParticipation
    );
    
    // Approval threshold
    let approval_rate = case.approve_votes as f64 / total_votes as f64;
    case.status = if approval_rate >= APPROVAL_THRESHOLD {
        CaseStatus::Approved
    } else {
        CaseStatus::Rejected
    };
    
    Ok(())
}
}

