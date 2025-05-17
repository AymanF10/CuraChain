use anchor_lang::{prelude::*, solana_program::{self, rent::Rent/*, system_program system_instruction*/}};

use solana_program::pubkey::Pubkey;

use crate::states::{contexts::*, errors::*, constants::*, events::*};




pub fn admin_override_case(ctx: Context<AdminOverrideCase>, case_id: String, is_verified: bool) -> Result<()> {
    let patient_case = &mut ctx.accounts.patient_case;

    // Only allow after 10 days from submission time
    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= patient_case.submission_time + ALLOWED_VERIFICATION_TIME as i64, 
        CuraChainError::VerifiersVerificationActive
    );

    // Case shouldn't already be verified
    require!(!patient_case.is_verified, CuraChainError::CaseAlreadyVerified);

    // Set the verification status based on admin decision
    patient_case.is_verified = is_verified;

    msg!("[ADMIN OVERRIDE] Case ID: {}", case_id);
    msg!("[ADMIN OVERRIDE] Patient case: {}", ctx.accounts.patient_case.key());
    msg!("[ADMIN OVERRIDE] Verification status set to: {}", is_verified);

    // If the admin approves the case, create an escrow account
    if is_verified {
        create_escrow_pda(ctx)?;
    }

    // Emit event for tracking admin override actions
    let message = format!(
        "Admin has overridden verification for case {}: {}",
        case_id,
        if is_verified { "APPROVED" } else { "REJECTED" }
    );
    
    let current_time = Clock::get()?.unix_timestamp;
    emit!(
        PatientCaseVerificationStatus{
            message,
            case_id,
            is_verified,
            timestamp: current_time,
        }
    );
    
    Ok(())
}


fn create_escrow_pda(ctx: Context<AdminOverrideCase>) -> Result<()> {
    
    let patient_case_key = ctx.accounts.patient_case.key();
    let case_id_lookup = &mut ctx.accounts.case_lookup;

    // Get Escrow PDA address using find_program_address
    let (patient_escrow_pda, _patient_escrow_bump) = Pubkey::find_program_address(
        &[b"patient_escrow", ctx.accounts.patient_case.case_id.as_bytes(), patient_case_key.as_ref()],
        ctx.program_id
    );

    // Verify passed PDA account matches derived one
    require!(
        *ctx.accounts.patient_escrow.key == patient_escrow_pda, CuraChainError::InvalidEscrowPDA
    );
    
    // Store the patient_escrow PDA bump in the case_lookup 
    case_id_lookup.patient_escrow_bump = _patient_escrow_bump;

    let rent = Rent::get()?;
    let space = 0;
    let lamports = rent.minimum_balance(space);

    // Create the Escrow PDA Account, setting system_program as owner
    let create_escrow_ix = solana_program::system_instruction::create_account(
        &ctx.accounts.admin.key(),
        &patient_escrow_pda,
        lamports,
        0,
        &solana_program::system_program::ID,
    );

    let accounts_needed = &[
        ctx.accounts.admin.to_account_info(),
        ctx.accounts.patient_escrow.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    ];

    let seeds = &[
        b"patient_escrow",
        ctx.accounts.patient_case.case_id.as_bytes().as_ref(),
        patient_case_key.as_ref(),
        &[_patient_escrow_bump],
    ];

    let signer_seeds = &[&seeds[..]];

    solana_program::program::invoke_signed(
        &create_escrow_ix,
        accounts_needed,
        signer_seeds
    )?;

    // Escrow PDA creation will throw an error if it fails
    Ok(())
}