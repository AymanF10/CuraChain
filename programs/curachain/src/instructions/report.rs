use anchor_lang::prelude::*;
use crate::{
    constants::{CASE_SEED, ESCROW_SEED},
    state::{PatientCase, EscrowPDA}
};

#[derive(Accounts)]
pub struct GenerateReport<'info> {
    #[account(seeds = [CASE_SEED, &patient_case.patient.key().as_ref()], bump)]
    pub patient_case: Account<'info, PatientCase>,
    
    #[account(seeds = [ESCROW_SEED, &patient_case.case_id.to_le_bytes()], bump)]
    pub escrow: Account<'info, EscrowPDA>,
}
impl<'info>GenerateReport<'info> {
pub fn handler(ctx: Context<GenerateReport>) -> Result<()> {
    let case = &ctx.accounts.patient_case;
    let escrow = &ctx.accounts.escrow;
    
    msg!(
        "Case Report - ID: {}, Status: {:?}, Raised: {} SOL",
        case.case_id,
        case.status,
        escrow.amount
    );
    Ok(())
}
}
