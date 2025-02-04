use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut)]
    pub patient_case: Account<'info, crate::state::PatientCase>,
    #[account(mut, constraint = patient_case.verified)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub hospital_token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
    let patient_case = &ctx.accounts.patient_case;

    // Transfer SPL-Tokens from escrow to hospital
    let transfer_instruction = Transfer {
        from: ctx.accounts.escrow_token_account.to_account_info(),
        to: ctx.accounts.hospital_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
    );
    token::transfer(cpi_ctx, ctx.accounts.escrow_token_account.amount)?;

    Ok(())
}