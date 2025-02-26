use anchor_lang::prelude::*;
use crate::{
    constants::*,
    state::{VerifierRegistry, CaseCounter},
    errors::ErrorCode
};

#[derive(Accounts)]
#[instruction(verifier_pubkey: Pubkey, verifier_type: String)]
pub struct WhitelistVerifier<'info> {
    #[account(
        init,
        payer = admin,
        space = VerifierRegistry::LEN,
        seeds = [VERIFIER_SEED, verifier_pubkey.as_ref()],
        bump
    )]
    pub verifier_registry: Account<'info, VerifierRegistry>,
    
    #[account(mut, address = case_counter.admin @ ErrorCode::Unauthorized)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [COUNTER_SEED],
        bump,
    )]
    pub case_counter: Account<'info, CaseCounter>,
    pub system_program: Program<'info, System>,
}
impl<'info>WhitelistVerifier<'info> {
pub fn handler(
    ctx: Context<WhitelistVerifier>,
    verifier_pubkey: Pubkey,
    verifier_type: String
) -> Result<()> {
    let registry = &mut ctx.accounts.verifier_registry;
    registry.verifier_pubkey = verifier_pubkey;
    registry.verifier_type = verifier_type;
    registry.is_verified = true;
    Ok(())
}
}
