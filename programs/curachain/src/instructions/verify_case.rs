use anchor_lang::prelude::*;
use crate::{constants::*, state::{PatientCase, VerifierRegistry, VerificationPDA}, errors::ErrorCode};

#[derive(Accounts)]
pub struct VerifyCase<'info> {
    #[account(mut)]
    pub patient_case: Account<'info, PatientCase>,
    
    #[account(
        seeds = [VERIFIER_SEED, verifier.key().as_ref()],
        bump,
        constraint = verifier_registry.is_verified @ ErrorCode::VerifierNotWhitelisted
    )]
    pub verifier_registry: Account<'info, VerifierRegistry>,
    
    #[account(mut)]
    pub verifier: Signer<'info>,

    #[account(
        init,
        payer = verifier,
        space = VerificationPDA::LEN,
        seeds = [b"verification", patient_case.key().as_ref(), verifier.key().as_ref()],
        bump
    )]
    pub verification_pda: Account<'info, VerificationPDA>,
    
    pub system_program: Program<'info, System>,
}
impl<'info>VerifyCase<'info> {
pub fn handler(ctx: Context<VerifyCase>, vote: bool) -> Result<()> {
    let case = &mut ctx.accounts.patient_case;
    let verification = &mut ctx.accounts.verification_pda;
    
    require!(
        Clock::get()?.unix_timestamp - case.timestamp <= VERIFICATION_PERIOD,
        ErrorCode::VerificationTimeout
    );
    
    verification.case_id = case.case_id;
    verification.verifier = *ctx.accounts.verifier.key;
    verification.vote = vote;
    verification.verification_timestamp = Clock::get()?.unix_timestamp;

    match vote {
        true => case.approve_votes = case.approve_votes.checked_add(1).unwrap(),
        false => case.reject_votes = case.reject_votes.checked_add(1).unwrap(),
    }
    
    Ok(())
}
}