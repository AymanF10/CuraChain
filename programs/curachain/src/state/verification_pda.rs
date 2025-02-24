use anchor_lang::prelude::*;

#[account]
pub struct VerificationPDA {
    pub case_id: u64,
    pub verifier: Pubkey,
    pub vote: bool,
    pub verification_timestamp: i64,
}

impl VerificationPDA {
    pub const LEN: usize = 8 + 32 + 1 + 8;
}