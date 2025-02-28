use anchor_lang::prelude::*;

#[account]
pub struct DonationPDA {
    pub donor: Pubkey,
    pub amount: u64,
    pub case_id: u64,
}

impl DonationPDA {
    pub const LEN: usize = 32 + 8 + 8 + 8;
}
