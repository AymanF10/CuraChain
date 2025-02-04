use anchor_lang::prelude::*;
use crate::state::ProgramData;

#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(init, payer = authority, space = 8 + 8)]
    pub program_data: Account<'info, ProgramData>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_program(ctx: Context<InitializeProgram>) -> Result<()> {
    let program_data = &mut ctx.accounts.program_data;
    program_data.is_initialized = true;
    Ok(())
}