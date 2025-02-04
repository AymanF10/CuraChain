use anchor_lang::prelude::*;
use crate::state::{PatientCase, ESCROW_SEED};

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut)]
    pub patient_case: Account<'info, PatientCase>,
    #[account(mut, constraint = patient_case.verified)]
    pub escrow_account: AccountInfo<'info>,
    #[account(mut)]
    pub hospital_wallet: AccountInfo<'info>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
    let patient_case = &ctx.accounts.patient_case;

    // Transfer funds from escrow to hospital wallet
    let escrow_balance = ctx.accounts.escrow_account.lamports();
    **ctx.accounts.escrow_account.lamports.borrow_mut() -= escrow_balance;
    **ctx.accounts.hospital_wallet.lamports.borrow_mut() += escrow_balance;

    Ok(())
}