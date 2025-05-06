use anchor_lang::{prelude::*, solana_program::{self, rent::Rent}};

use solana_program::pubkey::Pubkey;

use crate::states::{constants::SCALE, contexts::*, errors::*, PatientCaseVerificationStatus};

pub fn approve_patient_case(ctx: Context<VerifyPatientCase>, case_id: String, is_yes: bool) -> Result<()> {
    // Get the accounts under this context
    let patient_details = &mut ctx.accounts.patient_case;
    let verifier_to_vote = ctx.accounts.verifier_account.key(); // Use verifier_account (PDA) instead of verifier
    let total_verifiers = ctx.accounts.verifiers_list.all_verifiers.len();

    // Check if voting period has expired (10 days = 864000 seconds)
    let now = Clock::get()?.unix_timestamp;
    require!(
        now <= patient_details.submission_timestamp + 864_000,
        CuraChainError::VotingPeriodExpired
    );

    // Check that patient case has not been already verified
    require!(patient_details.is_verified == false, CuraChainError::CaseAlreadyVerified);

    // Check if verifier has already voted on this particular case
    require!(
        !patient_details.voted_verifiers.contains(&verifier_to_vote),
        CuraChainError::VerifierAlreadyVoted
    );

    // Record the respective votes
    match is_yes {
        true => patient_details.verification_yes_votes = patient_details.verification_yes_votes.checked_add(1).ok_or(CuraChainError::OverflowError)?,
        false => patient_details.verification_no_votes = patient_details.verification_no_votes.checked_add(1).ok_or(CuraChainError::OverflowError)?,
    };

    // Add the verifier PDA to the voted verifiers list
    patient_details.voted_verifiers.push(verifier_to_vote);

    // Get the total votes
    let total_votes = patient_details.verification_yes_votes.checked_add(patient_details.verification_no_votes).ok_or(CuraChainError::OverflowError)?;

    // Type cast both total_votes and total_verifiers to u32 and SCALE to avoid overflow and precision loss
    let total_votes_u32_scaled = (total_votes as u32).checked_mul(SCALE).ok_or(CuraChainError::OverflowError)?;
    let total_verifiers_u32_scaled = (total_verifiers as u32).checked_mul(SCALE).ok_or(CuraChainError::OverflowError)?;

    // Get half verifiers
    let half_verifiers_scaled = total_verifiers_u32_scaled.checked_mul(50).ok_or(CuraChainError::OverflowError)?
        .checked_div(100).ok_or(CuraChainError::OverflowError)?;
    
    // If total votes is >= 50% of total verifiers, check verification threshold
    if total_votes_u32_scaled >= half_verifiers_scaled {
        // Check if yes votes are >= 70% of total votes
        let approval_threshold_70_scaled = total_votes_u32_scaled.checked_mul(70).ok_or(CuraChainError::OverflowError)?
            .checked_div(100).ok_or(CuraChainError::OverflowError)?;
        let yes_votes_scaled = (patient_details.verification_yes_votes as u32).checked_mul(SCALE).ok_or(CuraChainError::OverflowError)?;

        if yes_votes_scaled >= approval_threshold_70_scaled {
            patient_details.is_verified = true;

            // Create the Patient Escrow PDA Account
            create_escrow_pda(ctx)?;

            // Emit verification event
            let message = format!("Patient Case With ID, {} has successfully been verified!!!", case_id);
            let current_time = Clock::get()?.unix_timestamp;
            emit!(
                PatientCaseVerificationStatus {
                    message,
                    case_id,
                    is_verified: true,
                    timestamp: current_time,
                }
            );
        } else {
            // Keep the patient case as unverified
            patient_details.is_verified = false;
        }
    }

    Ok(())
}

fn create_escrow_pda(ctx: Context<VerifyPatientCase>) -> Result<()> {
    let patient_case_key = ctx.accounts.patient_case.key();
    let case_id_lookup = &mut ctx.accounts.case_lookup;

    // Get Escrow PDA address using find_program_address
    let (patient_escrow_pda, _patient_escrow_bump) = Pubkey::find_program_address(
        &[b"patient_escrow", ctx.accounts.patient_case.case_id.as_bytes(), patient_case_key.as_ref()],
        ctx.program_id
    );

    // Verify passed PDA account matches derived one
    require!(
        *ctx.accounts.patient_escrow.key == patient_escrow_pda,
        CuraChainError::InvalidEscrowPDA
    );
    
    // Store the patient_escrow pda bump in case_lookup
    case_id_lookup.patient_escrow_bump = _patient_escrow_bump;

    let rent = Rent::get()?;
    let space = 0;
    let lamports = rent.minimum_balance(space);

    // Create the Escrow PDA Account, setting program_id as owner
    let create_escrow_ix = solana_program::system_instruction::create_account(
        &ctx.accounts.verifier.key(),
        &patient_escrow_pda,
        lamports,
        0,
        &solana_program::system_program::ID,
    );

    let accounts_needed = &[
        ctx.accounts.verifier.to_account_info(),
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