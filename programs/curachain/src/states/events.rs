use anchor_lang::prelude::*;

#[event]
pub struct InitializeAdmin {
    pub admin_address: Pubkey,
    pub timestamp: i64,
    pub active_status: bool,
    pub message: String,
}


#[event]
pub struct GlobalRegistryInitializeEvent {
    pub message: String,
}

#[event]
pub struct AddingNewVerifier {
    pub address: Pubkey,
    pub timestamp: i64,
    pub message: String,
}

#[event]
pub struct RemovingExistingVerifier {
    pub address: Pubkey,
    pub timestamp: i64,
    pub message: String,
}


#[event]
pub struct PatientCaseSubmission {
    pub message: String,
    pub description: String,
    pub case_id: String,
    pub total_needed_amount: u64,
    pub total_raised: u64,
    pub link_to_records: String,
    pub is_verified: bool,
    pub timestamp: i64,
}


#[event]
pub struct PatientCaseVerificationStatus {
    pub message: String,
    pub case_id: String,
    pub is_verified: bool,
    pub timestamp: i64,
}


#[event]
pub struct CloseRejectedPatientCase {
    pub message: String,
    pub case_id: String,
    pub timestamp: i64,
}

#[event]
pub struct DonationsMade {
    pub message: String,
    pub donor_address: Pubkey,
    pub donated_amount: u64,
    pub case_id: String,
    pub timestamp: i64,
}


#[event]
pub struct ReleaseOfFunds {
    pub message: String,
    pub treatment_address: Pubkey,
    pub transferred_amount: u64,
    pub case_id: String,
    pub timestamp: i64,
}