use anchor_lang::{prelude::*, solana_program::{self, program_pack::Pack, rent::Rent}};
use anchor_spl::{associated_token::{create_idempotent, get_associated_token_address, Create}, token::spl_token::state::Mint, token_interface::{transfer_checked, TransferChecked}};

use crate::states::{contexts::*, errors::*, ReleaseOfFunds};

pub fn release_funds<'info>(ctx: Context<'_, '_, '_, 'info, ReleaseFunds<'info>>, case_id: String, proposal_index: u64) -> Result<()> {
   
    // ENSURE RELEASE PROPOSAL IS APPROVED
    require!(ctx.accounts.proposal.approved == true, CuraChainError::ProposalNotApproved);
    require!(ctx.accounts.proposal.executed == false, CuraChainError::ProposalAlreadyExecuted);
    require!(ctx.accounts.proposal.case_id == case_id, CuraChainError::NoProposalMade);
    require!(ctx.accounts.proposal.proposal_index == proposal_index, CuraChainError::InvalidProposalIndex);


    // We Get The Escrow Balance Including Rent-exempt
    let total_escrow_balance = ctx.accounts.patient_escrow.lamports();
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(0);

    let actual_escrow_balance;
    // We Intend To Keep Patient Escrow Even When Case Is Fully Funded;
    // We can always close the account later
    actual_escrow_balance = total_escrow_balance.checked_sub(rent_lamports).ok_or(CuraChainError::UnderflowError)?;
    require!(actual_escrow_balance > 0, CuraChainError::NonZeroAmount);

   
    //  ...............          SET UP FOR SOL TRANSFER VIA LOW-LEVEL SOLANA CALL         .............   //
    // ----------  ONLY TRANSFER IF THERE WAS A SOL DONATION  ---------------- //

    if ctx.accounts.patient_case.total_sol_raised > 0 {

        let patient_case_key = &ctx.accounts.patient_case.key();
 
        let seeds = &[
            b"patient_escrow",
            case_id.as_bytes().as_ref(),
            patient_case_key.as_ref(),
            &[ctx.accounts.case_lookup.patient_escrow_bump]
        ];

        let signer_seeds = &[&seeds[..]];
 
        let transfer_ix = solana_program::system_instruction::transfer(
            &ctx.accounts.patient_escrow.key(),
            &ctx.accounts.facility_address.key(),
            actual_escrow_balance
        );
 
        solana_program::program::invoke_signed(
            &transfer_ix,
            &[
                ctx.accounts.patient_escrow.to_account_info().clone(),
                ctx.accounts.facility_address.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info()
            ],
            signer_seeds
        )?;

         // Update Patient Case With This Transferred Amount Only When There Was A Sol Transfer
        ctx.accounts.patient_case.total_sol_raised = ctx.accounts.patient_case.total_sol_raised
            .checked_sub(actual_escrow_balance).ok_or(CuraChainError::UnderflowError)?;

    }

    // ------- SPL TOKENS TRANSFER TO TREATMENT FACILITY ATA IF THERE WERE SPL DONATIONS MADE  ------- //
    if ctx.accounts.patient_case.spl_donations.len() > 0 {

        // Iterate Through The Spl Donations, And For Each, Create A Facility ATA if it doesn't exist, and
        // Transfer The Donated Tokens To The Facility ATA
        let donations_size = ctx.accounts.patient_case.spl_donations.len();
        for spl_donation in 0..donations_size {

            require!(ctx.remaining_accounts.len() >= donations_size * 3, CuraChainError::InvalidMintsLength);

            let token_mint_info = &ctx.remaining_accounts[spl_donation * 3 + 0];
            let patient_token_vault = &ctx.remaining_accounts[spl_donation * 3 + 1];
            let facility_token_ata = &ctx.remaining_accounts[spl_donation * 3 + 2];

            let decimals = Mint::unpack(&token_mint_info.try_borrow_data()?)?.decimals;

            let each_spl_donation = &mut ctx.accounts.patient_case.spl_donations[spl_donation];

            require!(token_mint_info.key() == each_spl_donation.mint, CuraChainError::InvalidRemainingMints);
            require!(patient_token_vault.key() == each_spl_donation.patient_token_vault, CuraChainError::InvalidRemainingVaults);
            // 1. Get Patient Token Vault That Holds The Donated Tokens
            // 2. Create Facility ATA for That Token If It Doesn't Exist
            // 3. Initiate The Transfer From The Patient Token Vault to Facility ATA
            let (patient_vault, _vault_bump) = Pubkey::find_program_address(
                &[
                    b"patient_token_vault",
                    case_id.as_bytes().as_ref(),
                    ctx.accounts.patient_escrow.key().as_ref(),
                    each_spl_donation.mint.as_ref()
                ],
                ctx.program_id
            );

            require!(patient_token_vault.key() == patient_vault, CuraChainError::InvalidRemainingVaults);

            // For No Explicit Check, We use the `create_idempotent` function which creates the ATA if 
            // it doesn't exist, and does nothing if it exists, just like the init_if_needed anchor constraint
            let facility_ata = get_associated_token_address(
                &ctx.accounts.facility_address.key(),
                &token_mint_info.key()
            );
            
            let required_accounts = Create {
                payer: ctx.accounts.transfer_authority.to_account_info(),
                associated_token: facility_token_ata.clone(),
                authority: ctx.accounts.facility_address.to_account_info(),
                mint: token_mint_info.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info()
            };
            let token_cpi = ctx.accounts.associated_token.to_account_info();
            let cpi_ctx = CpiContext::new(token_cpi, required_accounts);
            // Call The create_idempotent function
            create_idempotent(cpi_ctx)?;
            require!(facility_token_ata.key() == facility_ata, CuraChainError::MismatchedFacilityAtas);

            // After Creating, ====>>> Make Transfer 
            let transfer_accounts = TransferChecked {
                from: patient_token_vault.clone(),
                mint: token_mint_info.clone(),
                to: facility_token_ata.clone(),
                authority: ctx.accounts.multisig.to_account_info()
            };
            let transfer_program = ctx.accounts.token_program.to_account_info();
            let seeds = &[
            b"multisig",
            b"escrow-authority".as_ref(),
            &[ctx.accounts.multisig.multisig_bump]
        ];
            let multisig_seeds = &[&seeds[..]];
            let transfer_cpi = CpiContext::new_with_signer(transfer_program, transfer_accounts, multisig_seeds);
            transfer_checked(transfer_cpi, each_spl_donation.total_mint_amount, decimals)?;

            // Let's Update The Accounting of spl Donations on the Patient Case
            each_spl_donation.total_mint_amount = 0;
        }
    }

    // For total_amount_needed, only subtract the minimum of (actual_escrow_balance, total_amount_needed) to
    // prevent underflow
    let amount_to_subtract = std::cmp::min(actual_escrow_balance, ctx.accounts.patient_case.total_amount_needed);
    ctx.accounts.patient_case.total_amount_needed = ctx.accounts.patient_case.total_amount_needed
        .checked_sub(amount_to_subtract).ok_or(CuraChainError::UnderflowError)?;

    // Reset case funded flag if there is still more needed after partial release of funds
    if ctx.accounts.patient_case.total_amount_needed > 0 {
        ctx.accounts.patient_case.case_funded = false;
    }

    // Mark Proposal As Executed To Prevent Replaying
    ctx.accounts.proposal.executed = true;


    // EMIT AN EVENT FOR THIS INSTRUCTION ON-CHAIN ANYTIME THERE IS A RELEASE OF FUNDS
    let current_time = Clock::get()?.unix_timestamp;
    let message = format!("Contributed Funds of amount {} has been released for patient case ID ,{} at time of ,{}",
         actual_escrow_balance, case_id, current_time);

    emit!(
        ReleaseOfFunds{
            message,
            treatment_address: ctx.accounts.facility_address.key(),
            transferred_amount: actual_escrow_balance,
            case_id: case_id,
            timestamp: current_time
        }
    );


    Ok(())
}