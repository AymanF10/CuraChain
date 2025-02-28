use anchor_lang::prelude::*;
use crate::{constants::*, state::{PatientCase, EscrowPDA, CaseStatus}, errors::ErrorCode};

#[derive(Accounts)]
#[instruction(case_id: u64)]
pub struct CreateEscrow<'info> {
    #[account(
        mut,
        seeds = [CASE_SEED, patient_case.patient.as_ref()],
        bump,
        constraint = patient_case.status == CaseStatus::Approved @ ErrorCode::CaseNotApproved
    )]
    pub patient_case: Account<'info, PatientCase>,
    
    #[account(
        init,
        payer = trusted_entity,
        space = EscrowPDA::LEN,
        seeds = [ESCROW_SEED, &case_id.to_le_bytes()],
        bump
    )]
    pub escrow: Account<'info, EscrowPDA>,
    
    #[account(mut)]
    pub trusted_entity: Signer<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info>CreateEscrow<'info> {
pub fn handler(ctx: Context<CreateEscrow>, case_id: u64) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;
    escrow.case_id = case_id;
    escrow.trusted_entity = *ctx.accounts.trusted_entity.key;
    escrow.amount = 0;
    Ok(())
}
}

