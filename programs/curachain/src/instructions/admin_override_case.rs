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

    msg!("[ADMIN OVERRIDE] case_id: {}", case_id);
    msg!("[ADMIN OVERRIDE] patient_case.key(): {}", ctx.accounts.patient_case.key());

    if is_verified {
        create_escrow_pda_admin(ctx)?;
    }

    let message = if is_verified {
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

fn create_escrow_pda_admin(ctx: Context<AdminOverrideCase>) -> Result<()> {
    use anchor_lang::solana_program::{self, pubkey::Pubkey, rent::Rent};
    let patient_case_key = ctx.accounts.patient_case.key();
    let case_id = ctx.accounts.patient_case.case_id.clone();
    let (patient_escrow_pda, _patient_escrow_bump) = Pubkey::find_program_address(
        &[b"patient_escrow", ctx.accounts.patient_case.case_id.as_bytes(), patient_case_key.as_ref()],
        ctx.program_id
    );
    msg!("[ESCROW PDA] case_id: {}", case_id);
    msg!("[ESCROW PDA] patient_case_key: {}", patient_case_key);
    msg!("[ESCROW PDA] derived patient_escrow_pda: {}", patient_escrow_pda);
    require!(
        *ctx.accounts.patient_escrow.key == patient_escrow_pda, CuraChainError::InvalidEscrowPDA
    );
    let case_id_lookup = &mut ctx.accounts.case_lookup;
    case_id_lookup.patient_escrow_bump = _patient_escrow_bump;
    let rent = Rent::get()?;
    let space = 0;
    let lamports = rent.minimum_balance(space);
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
    Ok(())
} 