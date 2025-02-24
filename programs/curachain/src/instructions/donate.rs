use anchor_lang::prelude::*;
use crate::{constants::*, state::{EscrowPDA, DonationPDA}, errors::ErrorCode};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Donate<'info> {
    #[account(mut, seeds = [ESCROW_SEED, &escrow.case_id.to_le_bytes()], bump)]
    pub escrow: Account<'info, EscrowPDA>,
    
    #[account(
        init,
        payer = donor,
        space = DonationPDA::LEN,
        seeds = [DONATION_SEED, donor.key().as_ref(), &escrow.case_id.to_le_bytes()],
        bump
    )]
    pub donation: Account<'info, DonationPDA>,
    
    #[account(mut)]
    pub donor: Signer<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info>Donate<'info> {
pub fn handler(ctx: Context<Donate>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidDonationAmount);
    
    let escrow = &mut ctx.accounts.escrow;
    let donation = &mut ctx.accounts.donation;
    let donor = &mut ctx.accounts.donor;
    
    **escrow.to_account_info().try_borrow_mut_lamports()? += amount;
    **donor.to_account_info().try_borrow_mut_lamports()? -= amount;
    
    escrow.amount += amount;
    donation.amount = amount;
    donation.donor = *donor.key;
    donation.case_id = escrow.case_id;
    
    Ok(())
}
}