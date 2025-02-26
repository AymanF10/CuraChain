use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct CaseCounter {
    pub count: u64,
    pub admin: Pubkey,
}

impl CaseCounter {
    pub const LEN: usize = 8 + 32;
    
    pub fn initialize(&mut self, admin: Pubkey) -> Result<()> {
        self.count = 0;
        self.admin = admin;
        Ok(())
    }
}