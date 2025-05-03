use anchor_lang::{prelude::*, solana_program::{self, rent::Rent}};
use anchor_spl::token::{self, Transfer as SplTransfer};

use crate::states::{contexts::*, errors::*, ReleaseOfFunds};

pub fn release_funds(ctx: Context<ReleaseFunds>, case_id: String) -> Result<()> {
    // Let's get the necessary accounts
    let patient_escrow = &mut ctx.accounts.patient_escrow;
    let patient_case = &mut ctx.accounts.patient_case;
    let verifiers_registry = &ctx.accounts.verifiers_list;
    let treatment_address = &mut ctx.accounts.facility_address;
    let case_lookup = &ctx.accounts.case_lookup;


    //Let's validate that the PDAs of the signers are actual verifiers from the registry
    require!(verifiers_registry.all_verifiers.contains(&ctx.accounts.verifier1_pda.key()) && 
        verifiers_registry.all_verifiers.contains(&ctx.accounts.verifier2_pda.key()) && 
        verifiers_registry.all_verifiers.contains(&ctx.accounts.verifier3_pda.key()), 
        CuraChainError::VerifierNotFound);

    // We Get The Escrow Balance Including Rent-exempt
    let total_escrow_balance = patient_escrow.lamports();
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(0);

    let mut actual_escrow_balance = 0u64;
    let mut close_account = false;

    // Only process SOL release if the escrow is funded
    if total_escrow_balance > rent_lamports {
        if patient_case.case_funded == true {
            actual_escrow_balance = total_escrow_balance;
            close_account = true;
        } else {
            actual_escrow_balance = total_escrow_balance.checked_sub(rent_lamports).ok_or(CuraChainError::UnderflowError)?;
        }

        require!(actual_escrow_balance > 0, CuraChainError::NonZeroAmount);

        // SET UP FOR TRANSFER VIA LOW-LEVEL SOLANA CALL 
        let patient_case_key = &patient_case.key();
        let seeds = &[
            b"patient_escrow",
            case_id.as_bytes().as_ref(),
            patient_case_key.as_ref(),
            &[case_lookup.patient_escrow_bump]
        ];
        let signer_seeds = &[&seeds[..]];
        let transfer_ix = solana_program::system_instruction::transfer(
            &patient_escrow.key(),
            &treatment_address.key(),
            actual_escrow_balance
        );
        solana_program::program::invoke_signed(
            &transfer_ix,
            &[
                patient_escrow.clone(),
                treatment_address.clone(),
                ctx.accounts.system_program.to_account_info()
            ],
            signer_seeds
        )?;

        // Only Check Remaining Balance When We Are Not Closing Account
        if !close_account {
            let final_balance_transfer = patient_escrow.lamports();
            require!(final_balance_transfer >= rent_lamports, 
                CuraChainError::InsufficientRentBalance);
        }

        // Update Patient Case With This Transferred Amount
        let prev_total_raised = patient_case.total_raised;
        patient_case.total_raised = patient_case.total_raised
            .checked_sub(actual_escrow_balance).ok_or(CuraChainError::UnderflowError)?;
        msg!("[DEBUG] SOL release: actual_escrow_balance={}, prev_total_raised={}, new_total_raised={}", actual_escrow_balance, prev_total_raised, patient_case.total_raised);

        // For total_amount_needed, only subtract the minimum of (actual_escrow_balance, total_amount_needed) to
        // prevent underflow
        let amount_to_subtract = std::cmp::min(actual_escrow_balance, patient_case.total_amount_needed);
        let prev_total_needed = patient_case.total_amount_needed;
        patient_case.total_amount_needed = patient_case.total_amount_needed
            .checked_sub(amount_to_subtract).ok_or(CuraChainError::UnderflowError)?;
        msg!("[DEBUG] SOL release: amount_to_subtract={}, prev_total_needed={}, new_total_needed={}", amount_to_subtract, prev_total_needed, patient_case.total_amount_needed);

        // Reset case funded flag if there is still more needed after partial release of funds
        if patient_case.total_raised < patient_case.total_amount_needed + 1_000_000 {
            patient_case.case_funded = false;
        }

        // Close account if fully funded
        if close_account {
            **patient_escrow.try_borrow_mut_lamports()? = 0;
            **treatment_address.try_borrow_mut_lamports()? = treatment_address
                .lamports()
                .checked_add(patient_escrow.lamports())
                .ok_or(CuraChainError::OverflowError)?;
        }
    }



    // EMIT AN EVENT FOR THIS INSTRUCTION ON-CHAIN ANYTIME THERE IS A RELEASE OF FUNDS
    let current_time = Clock::get()?.unix_timestamp;
    let message = format!("Contributed Funds of amount {} has been released for patient case ID ,{} at time of ,{}",
         actual_escrow_balance, case_id, current_time);

    emit!(
        ReleaseOfFunds{
            message,
            treatment_address: treatment_address.key(),
            transferred_amount: actual_escrow_balance,
            case_id: case_id,
            timestamp: current_time
        }
    );

   // SPL Token Release Logic (arbitrary slots via remaining_accounts)
    let remaining: &[AccountInfo] = unsafe { std::mem::transmute(&ctx.remaining_accounts[..]) };
    let num_slots = remaining.len() / 3; // 3 accounts per slot: patient_ata, facility_ata, mint
    let token_program = ctx.accounts.token_program.to_account_info();
    let (_multisig_pda_key, multisig_bump) = Pubkey::find_program_address(&[b"multisig"], ctx.program_id);
    let multisig_seeds: &[&[u8]] = &[b"multisig", &[multisig_bump]];
    let signer_seeds: &[&[&[u8]]] = &[multisig_seeds];
    for i in 0..num_slots {
        let patient_ata = &remaining[i * 3];
        let facility_ata = &remaining[i * 3 + 1];
        let mint = &remaining[i * 3 + 2];
        // Find the matching spl_donation record by mint
        let spl_index = patient_case.spl_donations.iter().position(|rec| rec.mint == *mint.key);
        if let Some(idx) = spl_index {
            let amount = patient_case.spl_donations[idx].amount;
            let total_raised = patient_case.total_raised;
            let total_amount_needed = patient_case.total_amount_needed;
            msg!("Before SPL release: total_raised={}, spl_donation.amount={}, amount={}", total_raised, amount, amount);
            if amount == 0 {
                msg!("Skipping SPL transfer for mint {}: amount is zero", mint.key);
                continue;
            }
            let spl_available = std::cmp::min(amount, total_raised);
            if spl_available == 0 {
                msg!("No SPL funds available for mint {}", mint.key);
                continue;
            }
            let cpi_accounts = SplTransfer {
                from: patient_ata.clone(),
                to: facility_ata.clone(),
                authority: ctx.accounts.multisig_pda.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
            msg!("Transferring {} tokens of mint {} from {} to {}", spl_available, mint.key, patient_ata.key, facility_ata.key);
            token::transfer(cpi_ctx, spl_available)?;
            // Now re-borrow mutably to update
            msg!("[DEBUG] SPL release pre-check: total_raised={}, spl_donation.amount={}, spl_available={}", total_raised, amount, spl_available);
            require!(total_raised >= spl_available, CuraChainError::UnderflowError);
            require!(amount >= spl_available, CuraChainError::UnderflowError);
            let prev_total_raised = patient_case.total_raised;
            let prev_spl_amount = amount;
            patient_case.total_raised = total_raised.checked_sub(spl_available).ok_or(CuraChainError::UnderflowError)?;
            patient_case.spl_donations[idx].amount = amount.checked_sub(spl_available).ok_or(CuraChainError::UnderflowError)?;
            let subtract_from_needed = std::cmp::min(spl_available, total_amount_needed);
            let prev_total_needed = total_amount_needed;
            patient_case.total_amount_needed = total_amount_needed.checked_sub(subtract_from_needed).ok_or(CuraChainError::UnderflowError)?;
            msg!("[DEBUG] SPL release: spl_available={}, prev_total_raised={}, new_total_raised={}, prev_spl_amount={}, new_spl_amount={}, subtract_from_needed={}, prev_total_needed={}, new_total_needed={}", spl_available, prev_total_raised, patient_case.total_raised, prev_spl_amount, patient_case.spl_donations[idx].amount, subtract_from_needed, prev_total_needed, patient_case.total_amount_needed);
            // Defensive: assert non-negative
            require!(patient_case.total_raised <= prev_total_raised, CuraChainError::UnderflowError);
            require!(patient_case.spl_donations[idx].amount <= prev_spl_amount, CuraChainError::UnderflowError);
            require!(patient_case.total_raised >= 0, CuraChainError::UnderflowError);
            require!(patient_case.spl_donations[idx].amount >= 0, CuraChainError::UnderflowError);
            if patient_case.total_raised < patient_case.total_amount_needed + 1_000_000 {
                patient_case.case_funded = false;
            }
        } else {
            msg!("No SPL donation record found for mint {}", mint.key);
        }
    }

    Ok(())
}