use anchor_lang::prelude::*;

#[account]
pub struct ProgramData {
    pub is_initialized: bool,
}

#[account]
pub struct PatientCase {
    pub patient_wallet: Pubkey,
    pub case_id: u64,
    pub medical_description: String,
    pub required_funds: u64,
    pub verified: bool,
    pub escrow_account: Pubkey,
    pub verified_by: Vec<Pubkey>,
    pub medical_records_tx_id: String,
    pub token_mint: Pubkey,
}

#[account]
pub struct Donation {
    pub donor_wallet: Pubkey,
    pub case_id: u64,
    pub amount: u64,
    pub token_mint: Pubkey,
}

#[account]
pub struct Verifier {
    pub verifier_wallet: Pubkey,
    pub is_active: bool,
}

pub const PATIENT_CASE_SEED: &[u8] = b"patient_case";
pub const ESCROW_SEED: &[u8] = b"escrow";
pub const VERIFIER_SEED: &[u8] = b"verifier";
pub const DONATION_SEED: &[u8] = b"donation";

pub const PATIENT_CASE_SIZE: usize = 32 + 8 + 500 + 8 + 1 + 32 + 32 * 10 + 32 + 500;
pub const DONATION_SIZE: usize = 32 + 8 + 8 + 32;
pub const VERIFIER_SIZE: usize = 32 + 1;