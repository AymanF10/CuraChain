use anchor_lang::prelude::*;
use crate::{
    constants::COUNTER_SEED,
    state::{DonorRecognition, CaseCounter},
    errors::ErrorCode
};

#[derive(Accounts)]
#[instruction(case_id: u64)]
pub struct RecognizeDonor<'info> {
    #[account(
        init,
        payer = admin,
        space = DonorRecognition::LEN,
        seeds = [
            b"recognition".as_ref(),
            &case_id.to_le_bytes(),
            donor.key.as_ref() 
        ],
        bump
    )]
    pub recognition: Account<'info, DonorRecognition>,

    #[account(mut, address = case_counter.admin @ ErrorCode::Unauthorized)]
    pub admin: Signer<'info>,
    
    /// CHECK: Safe as we're just using the key
    #[account(mut)]
    pub donor: AccountInfo<'info>,
    
    #[account(
        seeds = [COUNTER_SEED],
        bump,
    )]
    pub case_counter: Account<'info, CaseCounter>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> RecognizeDonor<'info> {
    pub fn handler(ctx: Context<RecognizeDonor>, recognition_type: String) -> Result<()> {
        let recognition = &mut ctx.accounts.recognition;
        recognition.donor = *ctx.accounts.donor.key;
        recognition.recognition_type = recognition_type;
        Ok(())
    }
}
