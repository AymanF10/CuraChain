use anchor_lang::{prelude::*, solana_program::{self, rent::Rent/*, system_program system_instruction*/}};

use solana_program::pubkey::Pubkey;

use crate::states::{contexts::*, errors::*, constants::*, events::*};




pub fn admin_override_case(ctx: Context<AdminOverrideCase>, case_id: String, is_verified: bool) -> Result<()> {
    let patient_case = &mut ctx.accounts.patient_case;

    // Only allow after 10 days
    let now = Clock::get()?.unix_timestamp;
    require!(now >= patient_case.submission_time + ALLOWED_VERIFICATION_TIME as i64 , CuraChainError::VerifiersVerificationActive);

    // Only if not already verified
    require!(!patient_case.is_verified, CuraChainError::CaseAlreadyVerified);

    patient_case.is_verified = is_verified;

    msg!("[ADMIN OVERRIDE] case_id: {}", case_id);
    msg!("[ADMIN OVERRIDE] patient_case.key(): {}", ctx.accounts.patient_case.key());

    if is_verified {
        create_escrow_pda(ctx)?;
    }

   // CATCHING THIS EVENT ON-CHAIN ANYTIME THIS INSTRUCTION OCCURS
    let message = format!("Patient Case With ID, {} has successfully been verified!!!", case_id);
    let current_time = Clock::get()?.unix_timestamp;
    emit!(
        PatientCaseVerificationStatus{
            message,
            case_id,
            is_verified: true,
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
    
    // Let's store the patient_escrow pda bump into a field in the case_lookup 
    case_id_lookup.patient_escrow_bump = _patient_escrow_bump;

    let rent = Rent::get()?;
    let space = 0;
    let lamports = rent.minimum_balance(space);

      //Create the Escrow PDA Account, setting program_id as owner
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

    // We Need To Ensure The Escrow Patient PDA account was created successfully
    // There will be an error from the create_account instruction if creation failed somehow.
   
    Ok(())


}