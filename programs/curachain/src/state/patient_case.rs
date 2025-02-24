use anchor_lang::prelude::*;

#[account]
pub struct PatientCase {
    pub patient: Pubkey,
    pub case_id: u64,
    pub encrypted_link: String,
    pub timestamp: i64,
    pub status: CaseStatus,
    pub approve_votes: u64,
    pub reject_votes: u64,
    pub total_verifiers: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum CaseStatus {
    Pending,
    Approved,
    Rejected,
}

impl PatientCase {
    pub const LEN: usize = 32 + 8 + 4 + 256 + 8 + 1 + 8 + 8 + 8;
}

