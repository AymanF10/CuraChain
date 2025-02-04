use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct CreateDonation<'info> {
    #[account(mut)]
    pub patient_case: Account<'info, crate::state::PatientCase>,
    #[account(init, payer = donor_wallet, space = 8 + 32 + 8 + 8 + 32)]
    pub donation: Account<'info, crate::state::Donation>,
    #[account(mut)]
    pub donor_wallet: Signer<'info>,
    #[account(mut)]
    pub donor_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateDonationParams {
    pub case_id: u64,
    pub amount: u64,
}

pub fn create_donation(
    ctx: Context<CreateDonation>,
    params: CreateDonationParams,
) -> Result<()> {
    let donation = &mut ctx.accounts.donation;
    donation.donor_wallet = ctx.accounts.donor_wallet.key();
    donation.case_id = params.case_id;
    donation.amount = params.amount;
    donation.token_mint = ctx.accounts.patient_case.token_mint;

    // Transfer SPL-Tokens from donor to escrow
    let transfer_instruction = Transfer {
        from: ctx.accounts.donor_token_account.to_account_info(),
        to: ctx.accounts.escrow_token_account.to_account_info(),
        authority: ctx.accounts.donor_wallet.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
    );
    token::transfer(cpi_ctx, params.amount)?;

    Ok(())
}