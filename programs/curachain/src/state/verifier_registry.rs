use anchor_lang::prelude::*;

#[account]
pub struct VerifierRegistry {
    pub verifier_pubkey: Pubkey,
    pub verifier_type: String,
    pub is_verified: bool,
}

impl VerifierRegistry {
    pub const LEN: usize = 32 + 4 + 50 + 1;
}
