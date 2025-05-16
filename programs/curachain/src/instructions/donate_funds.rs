use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

use anchor_spl::token_interface::{TransferChecked, transfer_checked};

use crate::states::{contexts::*, constants::*, errors::*, accounts::*, events::*};


pub fn donate_spl(ctx: Context<SplDonation>, case_id: String, donation_token: Pubkey, amount_to_donate: u64) -> Result<()> {
    
    // Let's Get the Patient Escrow PDA, Patient Case and Donor PDAs
    let patient_case = &mut ctx.accounts.patient_case;
    let patient_escrow = &mut ctx.accounts.patient_escrow;
    let donor_info = &mut ctx.accounts.donor_account;

    // Check to ensure if case is verified or not.
    require!(patient_case.is_verified == true, CuraChainError::UnverifiedCase);

    let donor = &ctx.accounts.donor;


    // We Need To Prevent Overfunding of a case
    require!(patient_case.case_funded == false, CuraChainError::CaseFullyFunded);

    require!(patient_escrow.try_lamports()? >= 890880, CuraChainError::EscrowNotExist);

    // We have already checked for valid case_id. Ensure non-zero amount
    require!(amount_to_donate > 0, CuraChainError::NonZeroAmount);

    // ------------------------           MATCH DONATION TOKEN TYPE, AND SET UP RELEVANT CPIs     ----------------------//

    

                // For SPL Token, perform Transfer From Donor's ATA to Patient Token Vault via CPI
    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.donor_ata.to_account_info(),
        mint: ctx.accounts.donation_token.to_account_info(),
        to: ctx.accounts.patient_token_vault.to_account_info(),
        authority: ctx.accounts.donor.to_account_info()
    };

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer_checked(cpi_ctx, amount_to_donate, ctx.accounts.donation_token.decimals)?;

    // Let's update or insert the spl donations mint and amount
    let mut found = false;
    for entry in patient_case.spl_donations.iter_mut() {
        if entry.mint == donation_token {
            entry.total_mint_amount = entry.total_mint_amount
                .checked_add(amount_to_donate)
                .ok_or(CuraChainError::OverflowError)?;
            found = true;
            break;
        }
    }
    // For A new Token
    if !found {
        patient_case.spl_donations.push(SplDonations{
            mint: donation_token,
            total_mint_amount: amount_to_donate,
            patient_token_vault: ctx.accounts.patient_token_vault.key()
        });
    }
   
    // If Case Has Reached Full Funding, Let's Reset The CaseFunded to true, to prevent further funds
    // We intend to allow a buffer of 1 SOL on all fundings
    // Known ISSUE: This is inefficient as we cannot sum SOL and Spl token together for tracking, as they have different prizes
    let mut grand_donations = patient_case.total_sol_raised;
    for each_spl in patient_case.spl_donations.iter() {
        grand_donations = grand_donations
            .checked_add(each_spl.total_mint_amount)
            .ok_or(CuraChainError::OverflowError)?;
    }
    if grand_donations >= patient_case.total_amount_needed + DONATION_BUFFER {
        patient_case.case_funded = true;
    }

    // DONOR INFO UPDATE
    // Let's update Donor Account
    let case_id_bytes = {
        let bytes = case_id.as_bytes();
        let mut arr = [0u8;8];
        arr.copy_from_slice(bytes);
        arr
    };
    donor_info.donor_address = donor.key();
    donor_info.donor_bump = ctx.bumps.donor_account;
    donor_info.total_donations = donor_info.total_donations.checked_add(amount_to_donate).ok_or(CuraChainError::OverflowError)?;
    if !donor_info.donated_cases.contains(&case_id_bytes) {
        donor_info.donated_cases.push(case_id_bytes);
    }

    // CATCHING THIS EVENT ON-CHAIN ANYTIME A DONATION IS MADE TO ANY CASE ID
    let message = format!("A Donor of address {} has contributed an amount of {} to patient case of ID {}", donor.key(), amount_to_donate, case_id);
    let current_time = Clock::get()?.unix_timestamp;

    emit!(DonationsMade {
        message,
        donor_address: donor.key(),
        donated_amount: amount_to_donate,
        case_id: case_id,
        timestamp: current_time
    });

    Ok(())
}



// SOL DONATION LOGIC
pub fn donate(ctx: Context<SolDonation>, case_id: String, amount_to_donate: u64) -> Result<()> {

    // Let's Get the Patient Escrow PDA, Patient Case and Donor PDAs
    let patient_case = &mut ctx.accounts.patient_case;
    let patient_escrow = &mut ctx.accounts.patient_escrow;
    let donor_info = &mut ctx.accounts.donor_account;

    // Check to ensure if case is verified or not.
    require!(patient_case.is_verified == true, CuraChainError::UnverifiedCase);

    let donor = &ctx.accounts.donor;


    // We Need To Prevent Overfunding of a case
    require!(patient_case.case_funded == false, CuraChainError::CaseFullyFunded);

    require!(patient_escrow.try_lamports()? >= 890880, CuraChainError::EscrowNotExist);

    // We have already checked for valid case_id. Ensure non-zero amount
    require!(amount_to_donate > 0, CuraChainError::NonZeroAmount);

    // Transfer Donor's SOL to Patient Escrow
    let cpi_program = ctx.accounts.system_program.to_account_info();

    let cpi_accounts = Transfer {
        from: donor.to_account_info(),
        to: patient_escrow.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer(cpi_ctx, amount_to_donate)?;

    // Let's Update the patient-case with these infos
    patient_case.total_sol_raised = patient_case.total_sol_raised.checked_add(amount_to_donate).ok_or(CuraChainError::OverflowError)?;
    
    // DONOR INFO UPDATE
    // Let's update Donor Account
    let case_id_bytes = {
        let bytes = case_id.as_bytes();
        let mut arr = [0u8;8];
        arr.copy_from_slice(bytes);
        arr
    };
    donor_info.donor_address = donor.key();
    donor_info.donor_bump = ctx.bumps.donor_account;
    donor_info.total_donations = donor_info.total_donations.checked_add(amount_to_donate).ok_or(CuraChainError::OverflowError)?;
    if !donor_info.donated_cases.contains(&case_id_bytes) {
        donor_info.donated_cases.push(case_id_bytes);
    }
    Ok(())
}