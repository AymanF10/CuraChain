use anchor_lang::prelude::*;

pub mod instructions;
use crate::instructions::*;

pub mod state;
pub mod constants;
pub mod errors;

pub use recognize::*; 
declare_id!("GadKQ2cTG19siDDAmvVqdnTBKFrvMLSbU2x3hSXGU6G2");

#[program]
    pub mod curachain{
    use super::*;

    pub fn initialize_counter(ctx: Context<InitializeCounter>) -> Result<()> {
        instructions::initialize_counter::InitializeCounter::handler(ctx)
    }

    pub fn submit_patient_case(
        ctx: Context<SubmitPatientCase>,
        encrypted_link: String, case_id: u64
    ) -> Result<()> {
        instructions::submit_case::SubmitPatientCase::handler(ctx, encrypted_link, case_id)
    }

    pub fn whitelist_verifier(
        ctx: Context<WhitelistVerifier>,
        verifier_pubkey: Pubkey,
        verifier_type: String,
    ) -> Result<()> {
        instructions::whitelist::WhitelistVerifier::handler(ctx, verifier_pubkey, verifier_type)
    }

    pub fn verify_case(ctx: Context<VerifyCase>, vote: bool) -> Result<()> {
        instructions::verify_case::VerifyCase::handler(ctx, vote)
    }
    
    pub fn finalize_verification(
        ctx: Context<FinalizeVerification>,
        case_id: u64,
    ) -> Result<()> {
        instructions::finalize_verification::FinalizeVerification::handler(ctx, case_id)
    }

    pub fn create_escrow(ctx: Context<CreateEscrow>, case_id: u64) -> Result<()> {
        instructions::create_escrow::CreateEscrow::handler(ctx, case_id)
    }

    pub fn donate(ctx: Context<Donate>, amount: u64) -> Result<()> {
        instructions::donate::Donate::handler(ctx, amount)
    }

    pub fn recognize_donor(
        ctx: Context<RecognizeDonor>,
        recognition_type: String,
    ) -> Result<()> {
        instructions::recognize::RecognizeDonor::handler(ctx, recognition_type)
    }

    pub fn generate_report(ctx: Context<GenerateReport>) -> Result<()> {
        instructions::report::GenerateReport::handler(ctx)
    }

    pub fn track_status(ctx: Context<TrackStatus>) -> Result<()> {
        instructions::track_status::TrackStatus::handler(ctx)
    }

    pub fn check_compliance(ctx: Context<CheckCompliance>) -> Result<bool> {
        instructions::compliance::CheckCompliance::handler(ctx)
    }
}


//Program Id: GadKQ2cTG19siDDAmvVqdnTBKFrvMLSbU2x3hSXGU6G2
//Signature: 4aFVWfH97PQFwD1Wv7r9sN5SdQyd8ajHDGzYujCNRoAAxdb4S98dRxznMe6JNKWcpVBYMAfDQEiQpniUsPdDaqsK
//Deploy success
