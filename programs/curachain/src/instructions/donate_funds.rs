use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};
use anchor_spl::token_interface::{transfer_checked, TransferChecked, TokenInterface};
use crate::states::{contexts::*, errors::*, DonationsMade, SplDonationRecord};
use crate::states::accounts::DonorNftRecord;
use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, update_metadata_accounts_v2, UpdateMetadataAccountsV2};
use mpl_token_metadata::types::DataV2;

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

    // NFT minting logic (if all NFT accounts are provided)
    // Only mint if donor has not already received an NFT for this case
    if let (Some(nft_mint_info), Some(nft_ata_info), Some(metadata), Some(token_metadata_program), Some(update_authority_pda)) = (
        ctx.accounts.donor_nft_mint.as_ref(),
        ctx.accounts.donor_ata.as_ref(),
        ctx.accounts.donor_nft_metadata.as_ref(),
        ctx.accounts.token_metadata_program.as_ref(),
        ctx.accounts.update_authority_pda.as_ref(),
    ) {
        msg!("Starting NFT minting for donor: {}, case_id: {}", donor.key(), case_id);
        // Clone case_id for NFT logic to avoid move error
        let case_id_for_nft = case_id.clone();
        let mut case_id_bytes = [0u8; 10];
        let case_id_bytes_src = case_id_for_nft.as_bytes();
        case_id_bytes[..case_id_bytes_src.len().min(10)]
            .copy_from_slice(&case_id_bytes_src[..case_id_bytes_src.len().min(10)]);
        // Check if donor already has an NFT for this case
        let existing_nft = donor_info.nft_cases.iter_mut().find(|rec| rec.case_id == case_id_bytes);
        let mut new_total = amount_to_donate;
        if let Some(nft_record) = existing_nft {
            // Increment total_donated
            nft_record.total_donated = nft_record.total_donated
                .checked_add(amount_to_donate)
                .ok_or(CuraChainError::OverflowError)?;
            new_total = nft_record.total_donated;

            // Update the NFT metadata with the new total
            let new_uri = format!("https://arweave.net/your_image_uri?total_donated={}", new_total);
            let new_data = DataV2 {
                name: "CuraChain Donor NFT".to_string(),
                symbol: "CURA".to_string(),
                uri: new_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            };
            let cpi_accounts = UpdateMetadataAccountsV2 {
                metadata: metadata.to_account_info(),
                update_authority: update_authority_pda.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(token_metadata_program.to_account_info(), cpi_accounts);
            update_metadata_accounts_v2(
                cpi_ctx,
                Some(update_authority_pda.key()), // new_update_authority
                Some(new_data), // data
                None, // primary_sale_happened
                Some(true), // is_mutable
            )?;
            msg!("Updated NFT metadata for case_id: {} with new total_donated: {}", case_id, new_total);
        } else {
            use anchor_spl::token::{MintTo, mint_to};
            // Mint 1 NFT to the donor
            let cpi_accounts = MintTo {
                mint: nft_mint_info.to_account_info(),
                to: nft_ata_info.to_account_info(),
                authority: ctx.accounts.donor.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
            mint_to(cpi_ctx, 1)?;
            msg!("Minted NFT to donor: {}", ctx.accounts.donor.key());

            // Create the DataV2 struct
            let new_uri = format!("https://arweave.net/your_image_uri?total_donated={}", amount_to_donate);
            let data = DataV2 {
                name: "CuraChain Donor NFT".to_string(),
                symbol: "CURA".to_string(),
                uri: new_uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            };

            // Create the CPI accounts
            let metadata_accounts = CreateMetadataAccountsV3 {
                metadata: metadata.to_account_info(),
                mint: nft_mint_info.to_account_info(),
                mint_authority: ctx.accounts.donor.to_account_info(),
                payer: ctx.accounts.donor.to_account_info(),
                update_authority: update_authority_pda.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            };

            // Create the CPI context
            let metadata_ctx = CpiContext::new(
                token_metadata_program.to_account_info(),
                metadata_accounts,
            );

            // Call the function
            create_metadata_accounts_v3(
                metadata_ctx,
                data,
                true,  // is_mutable
                true,  // update_authority_is_signer
                None,  // collection_details
            )?;
            msg!("Created NFT metadata for case_id: {}", case_id);

            // Record that donor received NFT for this case
            donor_info.nft_cases.push(DonorNftRecord {
                case_id: case_id_bytes,
                mint: nft_mint_info.key(),
                total_donated: amount_to_donate,
            });
        }
    }

    Ok(())
}