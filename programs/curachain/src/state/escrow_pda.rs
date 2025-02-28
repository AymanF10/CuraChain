use anchor_lang::prelude::*;

#[account]
pub struct EscrowPDA {
    pub case_id: u64,
    pub trusted_entity: Pubkey,
    pub amount: u64,
}

impl EscrowPDA {
    pub const LEN: usize = 8 + 32 + 8 + 8;
}