use anchor_lang::prelude::*;
use crate::{
    constants::{COUNTER_SEED, ADMIN_PUBKEY},
    state::CaseCounter,
    errors::ErrorCode
};

#[derive(Accounts)]
pub struct InitializeCounter<'info> {
    #[account(
        init,
        payer = admin,
        space = CaseCounter::LEN,
        seeds = [COUNTER_SEED],
        bump
    )]
    pub case_counter: Account<'info, CaseCounter>,
    
    #[account(mut, address = ADMIN_PUBKEY @ ErrorCode::Unauthorized)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info>InitializeCounter<'info> {
pub fn handler(ctx: Context<InitializeCounter>) -> Result<()> {
    let counter = &mut ctx.accounts.case_counter;
    counter.initialize()?;
    Ok(())
}
}