use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct CaseCounter {
    pub count: u64,
}

impl CaseCounter {
    pub const LEN: usize = 8 + 8;
    
    pub fn initialize(&mut self) -> Result<()> {
        self.count = 0;
        Ok(())
    }
}

