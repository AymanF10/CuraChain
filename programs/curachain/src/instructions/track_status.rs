use anchor_lang::prelude::*;
use crate::{constants::CASE_SEED, state::PatientCase};

#[derive(Accounts)]
pub struct TrackStatus<'info> {
    #[account(seeds = [CASE_SEED, &patient_case.case_id.to_le_bytes()], bump)]
    pub patient_case: Account<'info, PatientCase>,
}
impl<'info>TrackStatus<'info> {
pub fn handler(ctx: Context<TrackStatus>) -> Result<()> {
    let case = &ctx.accounts.patient_case;
    msg!(
        "Case Status - ID: {}, Status: {:?}, Last Updated: {}",
        case.case_id,
        case.status,
        case.timestamp
    );
    Ok(())
}
}