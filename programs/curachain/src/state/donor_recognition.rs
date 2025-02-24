use anchor_lang::prelude::*;

#[account]
pub struct DonorRecognition {
    pub donor: Pubkey,
    pub recognition_type: String,
}

impl DonorRecognition {
    pub const LEN: usize = 32 + 4 + 50;
}