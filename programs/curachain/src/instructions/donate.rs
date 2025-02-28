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

impl<'info> Donate<'info> {
    pub fn handler(ctx: Context<Donate>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidDonationAmount);
        
        // Transfer lamports from donor to escrow securely
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.donor.to_account_info(),
                    to: ctx.accounts.escrow.to_account_info(),
                },
            ),
            amount,
        )?;

   let escrow = &mut ctx.accounts.escrow;
    escrow.amount = escrow.amount.checked_add(amount).unwrap();

        // Initialize donation account data
        let donation = &mut ctx.accounts.donation;
        donation.amount = amount;
        donation.donor = *ctx.accounts.donor.key;
        donation.case_id = ctx.accounts.escrow.case_id;
        
        Ok(())
    }
}
    

    