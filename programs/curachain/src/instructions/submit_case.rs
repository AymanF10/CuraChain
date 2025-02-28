use anchor_lang::prelude::*;
use crate::{
    constants::*,
    state::{PatientCase, CaseStatus}  
};
use crate::state::CaseCounter;

#[derive(Accounts)]
#[instruction(encrypted_link: String, case_id: u64)]
pub struct SubmitPatientCase<'info> {
    #[account(
        init,
        payer = patient,
        space = PatientCase::LEN,
        seeds = [CASE_SEED, patient.key().as_ref()],
        bump
    )]
    pub patient_case: Account<'info, PatientCase>,

    #[account(mut, seeds = [COUNTER_SEED], bump)] 
    pub case_counter: Account<'info, CaseCounter>,
    
    #[account(mut)]
    pub patient: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> SubmitPatientCase<'info> {
    pub fn handler(ctx: Context<SubmitPatientCase>, encrypted_link: String, case_id: u64) -> Result<()> {
        let case = &mut ctx.accounts.patient_case;
        let counter = &mut ctx.accounts.case_counter;
        
        case.case_id;
        case.patient = *ctx.accounts.patient.key;
        case.encrypted_link = encrypted_link;
        case.timestamp = Clock::get()?.unix_timestamp;
        case.status = CaseStatus::Pending;

        case.approve_votes = 0;
    case.reject_votes = 0;
    case.total_verifiers = 1; // Match the number of whitelisted verifiers
    
    counter.count = counter.count.checked_add(1).unwrap();

        //counter.count = counter.count.checked_add(1).unwrap();
        //case.total_verifiers = 1;
        //case.total_verifiers = 1;

        Ok(())
    }
}



    
    

