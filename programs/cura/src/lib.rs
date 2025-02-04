use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

// Import modules
mod state;
mod instructions;

// Re-export instructions for external use
pub use instructions::*;

// Program ID (replace with your actual program ID)
declare_id!("J9Wn6DjGSnSRvF7vxk9hhp6BWLbgoUrvvXvNiHtTnKH7");

#[program]
pub mod cura {
    use super::*;

    // Initialize the program (optional, can be removed if not needed)
    pub fn initialize_program(ctx: Context<InitializeProgram>) -> Result<()> {
        instructions::initialize::initialize_program(ctx)
    }

    // Create a patient case
    pub fn create_patient_case(
        ctx: Context<CreatePatientCase>,
        case_id: u64,
        medical_description: String,
        required_funds: u64,
        medical_records_tx_id: String,
        token_mint: Pubkey,
    ) -> Result<()> {
        instructions::create_patient_case::create_patient_case(
            ctx,
            case_id,
            medical_description,
            required_funds,
            medical_records_tx_id,
            token_mint,
        )
    }

    // Create a donation
    pub fn create_donation(
        ctx: Context<CreateDonation>,
        case_id: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::create_donation::create_donation(ctx, case_id, amount)
    }

    // Verify a patient case
    pub fn verify_case(ctx: Context<VerifyCase>) -> Result<()> {
        instructions::verify_case::verify_case(ctx)
    }

    // Withdraw funds to the hospital's wallet
    pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
        instructions::withdraw_funds::withdraw_funds(ctx)
    }
}

// Contexts for instructions
#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(init, payer = authority, space = 8 + 8)]
    pub program_data: Account<'info, state::ProgramData>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePatientCase<'info> {
    #[account(init, payer = patient_wallet, space = state::PATIENT_CASE_SIZE)]
    pub patient_case: Account<'info, state::PatientCase>,
    #[account(mut)]
    pub patient_wallet: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateDonation<'info> {
    #[account(mut)]
    pub patient_case: Account<'info, state::PatientCase>,
    #[account(init, payer = donor_wallet, space = 8 + 32 + 8 + 8 + 32)]
    pub donation: Account<'info, state::Donation>,
    #[account(mut)]
    pub donor_wallet: Signer<'info>,
    #[account(mut)]
    pub donor_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyCase<'info> {
    #[account(mut)]
    pub patient_case: Account<'info, state::PatientCase>,
    #[account(
        seeds = [state::VERIFIER_SEED, &verifier_wallet.key().to_bytes()],
        bump,
        constraint = verifier.is_active
    )]
    pub verifier: Account<'info, state::Verifier>,
    pub verifier_wallet: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut)]
    pub patient_case: Account<'info, state::PatientCase>,
    #[account(mut, constraint = patient_case.verified)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub hospital_token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}