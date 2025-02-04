use anchor_lang::prelude::*;
use crate::state::{PatientCase, PATIENT_CASE_SEED, ESCROW_SEED, PATIENT_CASE_SIZE};

#[derive(Accounts)]
pub struct CreatePatientCase<'info> {
    #[account(init, payer = patient_wallet, space = PATIENT_CASE_SIZE)]
    pub patient_case: Account<'info, PatientCase>,
    #[account(mut)]
    pub patient_wallet: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreatePatientCaseParams {
    pub case_id: u64,
    pub medical_description: String,
    pub required_funds: u64,
    pub medical_records_tx_id: String,
    pub token_mint: Pubkey,
}

pub fn create_patient_case(
    ctx: Context<CreatePatientCase>,
    params: CreatePatientCaseParams,
) -> Result<()> {
    let patient_case = &mut ctx.accounts.patient_case;
    patient_case.patient_wallet = ctx.accounts.patient_wallet.key();
    patient_case.case_id = params.case_id;
    patient_case.medical_description = params.medical_description;
    patient_case.required_funds = params.required_funds;
    patient_case.verified = false;
    patient_case.medical_records_tx_id = params.medical_records_tx_id;
    patient_case.token_mint = params.token_mint;
    patient_case.verified_by = Vec::new();

    // Create an escrow account for the patient case
    let escrow_account = Pubkey::create_program_address(
        &[ESCROW_SEED, &patient_case.case_id.to_le_bytes()],
        ctx.program_id,
    )?;
    patient_case.escrow_account = escrow_account;

    Ok(())
}