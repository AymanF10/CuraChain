use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use anchor_spl::token::{self, Transfer as SplTransfer};
use crate::states::{contexts::*, errors::*, DonationsMade, SplDonationRecord};

// The special constant for native SOL (11111111111111111111111111111111)
const NATIVE_MINT: &str = "11111111111111111111111111111111";

pub fn donate_funds_to_patient_escrow(ctx: Context<Donation>, case_id: String, amount_to_donate: u64, mint: Pubkey) -> Result<()> {
    require!(ctx.accounts.patient_case.is_verified == true, CuraChainError::UnverifiedCase);
    let patient_case = &mut ctx.accounts.patient_case;
    let donor_info = &mut ctx.accounts.donor_account;
    let donor = &ctx.accounts.donor;

    // Prevent overfunding
    require!(patient_case.case_funded == false, CuraChainError::CaseFullyFunded);
    require!(amount_to_donate > 0, CuraChainError::NonZeroAmount);

    // --- SOL DONATION LOGIC ---
    if mint.to_string() == NATIVE_MINT {
        let patient_escrow = &mut ctx.accounts.patient_escrow;
        require!(patient_escrow.try_lamports()? >= 890880, CuraChainError::EscrowNotExist);
        require!(donor.to_account_info().lamports() >= amount_to_donate, CuraChainError::InsufficientBalance);

        donor_info.donor_address = donor.key();
        donor_info.donor_bump = ctx.bumps.donor_account;
        donor_info.total_donations = donor_info.total_donations.checked_add(amount_to_donate).ok_or(CuraChainError::OverflowError)?;
        patient_case.total_raised = patient_case.total_raised.checked_add(amount_to_donate).ok_or(CuraChainError::OverflowError)?;

        // Donor sends SOL to the multisig-owned escrow account
        let cpi_program = ctx.accounts.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: donor.to_account_info(),
            to: patient_escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount_to_donate)?;
    } else {
        // --- SPL TOKEN DONATION LOGIC ---
        // Donor sends SPL tokens to the multisig-owned patient ATA
        let donor_ata = ctx.accounts.donor_ata.to_account_info();
        let patient_ata = ctx.accounts.patient_ata.to_account_info(); // This ATA must be owned by the multisig PDA
        let token_program = ctx.accounts.token_program.to_account_info();
        let donor = ctx.accounts.donor.to_account_info();

        // Donor is the authority for this transfer
        let cpi_accounts = SplTransfer {
            from: donor_ata,
            to: patient_ata,
            authority: donor.clone(),
        };
        let cpi_ctx = CpiContext::new(token_program, cpi_accounts);
        token::transfer(cpi_ctx, amount_to_donate)?;

        // Update spl_donations in PatientCase
        let mut found = false;
        for rec in &mut patient_case.spl_donations {
            if rec.mint == mint {
                rec.amount = rec.amount.checked_add(amount_to_donate).ok_or(CuraChainError::OverflowError)?;
                found = true;
                break;
            }
        }
        if !found {
            require!(patient_case.spl_donations.len() < 10, CuraChainError::OverflowError); // Respect max_len
            patient_case.spl_donations.push(SplDonationRecord { mint, amount: amount_to_donate });
        }
    }

    // If Case Has Reached Full Funding, Let's Reset The CaseFunded to true, to prevent further funds
    if patient_case.total_raised >= patient_case.total_amount_needed + 1000000 {
        patient_case.case_funded = true;
    }

    // Emit event
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