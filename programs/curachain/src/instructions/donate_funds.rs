use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use anchor_spl::token_interface::{transfer_checked, TransferChecked};
use crate::states::{contexts::*, errors::*, DonationsMade, SplDonationRecord};

const NATIVE_MINT: &str = "11111111111111111111111111111111";

pub fn donate_funds_to_patient_escrow(
    ctx: Context<Donation>,
    case_id: String,
    amount_to_donate: u64,
    mint: Pubkey,
) -> Result<()> {
    require!(ctx.accounts.patient_case.is_verified, CuraChainError::UnverifiedCase);
    let patient_case = &mut ctx.accounts.patient_case;
    let donor_info = &mut ctx.accounts.donor_account;
    let donor = &ctx.accounts.donor;

    require!(patient_case.case_funded == false, CuraChainError::CaseFullyFunded);
    require!(amount_to_donate > 0, CuraChainError::NonZeroAmount);

    msg!("Processing donation for case_id: {}, amount: {}, mint: {}", case_id, amount_to_donate, mint);

    if mint.to_string() == NATIVE_MINT {
        // SOL donation logic
        let patient_escrow = &mut ctx.accounts.patient_escrow;
        require!(patient_escrow.try_lamports()? >= 890880, CuraChainError::EscrowNotExist);
        require!(donor.to_account_info().lamports() >= amount_to_donate, CuraChainError::InsufficientBalance);

        donor_info.donor_address = donor.key();
        donor_info.donor_bump = ctx.bumps.donor_account;
        donor_info.total_donations = donor_info.total_donations
            .checked_add(amount_to_donate)
            .ok_or(CuraChainError::OverflowError)?;
        patient_case.total_raised = patient_case.total_raised
            .checked_add(amount_to_donate)
            .ok_or(CuraChainError::OverflowError)?;

        let cpi_program = ctx.accounts.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: donor.to_account_info(),
            to: patient_escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount_to_donate)?;
    } else {
        // SPL donation logic
        let donor_ata = ctx.accounts.donor_ata
            .as_ref()
            .ok_or(CuraChainError::MissingDonorAta)?;
        let patient_ata = ctx.accounts.patient_ata
            .as_ref()
            .ok_or(CuraChainError::MissingPatientAta)?;
        let mint_info = ctx.accounts.mint
            .as_ref()
            .ok_or(CuraChainError::InvalidMint)?;
        let mint_decimals = anchor_spl::token_interface::Mint::try_deserialize(
            &mut &mint_info.data.borrow()[..]
        )?.decimals;
        require!(donor_ata.mint == mint_info.key(), CuraChainError::InvalidMint);
        require!(donor_ata.owner == ctx.accounts.donor.key(), CuraChainError::InvalidDonor);
        require!(patient_ata.mint == mint_info.key(), CuraChainError::InvalidMint);
        require!(
            patient_ata.owner == ctx.accounts.multisig_pda.key(),
            CuraChainError::InvalidAuthority
        );
        let donor_ata_info = donor_ata.to_account_info();
        let patient_ata_info = patient_ata.to_account_info();
        let token_program = ctx.accounts.token_program.to_account_info();
        let donor = ctx.accounts.donor.to_account_info();
        require!(
            donor_ata.to_account_info().owner == &token_program.key(),
            CuraChainError::AccountOwnedByWrongProgram
        );
        let cpi_accounts = TransferChecked {
            from: donor_ata_info,
            to: patient_ata_info,
            authority: donor.clone(),
            mint: mint_info.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(token_program, cpi_accounts);
        transfer_checked(cpi_ctx, amount_to_donate, mint_decimals)?;

        let mut found = false;
        for rec in &mut patient_case.spl_donations {
            if rec.mint == mint_info.key() {
                rec.amount = rec.amount
                    .checked_add(amount_to_donate)
                    .ok_or(CuraChainError::OverflowError)?;
                found = true;
                break;
            }
        }
        if !found {
            require!(patient_case.spl_donations.len() < 10, CuraChainError::OverflowError);
            patient_case.spl_donations.push(SplDonationRecord {
                mint: mint_info.key(),
                amount: amount_to_donate,
            });
        }
        let new_total_raised = patient_case.total_raised
            .checked_add(amount_to_donate)
            .ok_or(CuraChainError::OverflowError)?;
        msg!(
            "[DEBUG] SPL donation: mint={}, amount_to_donate={}, prev_total_raised={}, new_total_raised={}",
            mint_info.key(),
            amount_to_donate,
            patient_case.total_raised,
            new_total_raised
        );
        patient_case.total_raised = new_total_raised;
    }

    if patient_case.total_raised >= patient_case.total_amount_needed + 1_000_000 {
        patient_case.case_funded = true;
    }

    let message = format!(
        "A Donor of address {} has contributed an amount of {} to patient case of ID {}",
        donor.key(),
        amount_to_donate,
        case_id.clone()
    );
    let current_time = Clock::get()?.unix_timestamp;
    emit!(DonationsMade {
        message,
        donor_address: donor.key(),
        donated_amount: amount_to_donate,
        case_id: case_id.clone(),
        timestamp: current_time
    });

    Ok(())
}