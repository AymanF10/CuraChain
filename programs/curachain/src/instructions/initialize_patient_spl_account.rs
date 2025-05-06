use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, Mint, initialize_account, InitializeAccount};
use crate::states::errors::CuraChainError;

// Constants
const TOKEN_ACCOUNT_SIZE: u64 = 165; // Standard size for a token account (TokenAccount::LEN from spl_token)

#[derive(Accounts)]
#[instruction(case_id: String, mint: Pubkey)]
pub struct InitializePatientSplAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub mint: InterfaceAccount<'info, Mint>,
    
    /// CHECK: The multisig PDA (authority/owner of the SPL token account)
    #[account(mut)]
    pub multisig_pda: AccountInfo<'info>,
    
    /// CHECK: The PDA for the SPL token account to be created
    #[account(
        mut,
        seeds = [b"patient_spl", case_id.as_bytes(), mint.key().as_ref(), multisig_pda.key().as_ref()],
        bump,
    )]
    pub patient_spl_ata: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_patient_spl_account(
    ctx: Context<InitializePatientSplAccount>,
    case_id: String,
    _mint: Pubkey,
) -> Result<()> {
    use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

    let multisig_pda = ctx.accounts.multisig_pda.key();
    let patient_spl_ata = ctx.accounts.patient_spl_ata.to_account_info();
    let mint = ctx.accounts.mint.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();
    let rent_sysvar = ctx.accounts.rent.to_account_info();
    let payer = ctx.accounts.payer.to_account_info();

    let seeds = &[b"patient_spl", case_id.as_bytes(), mint.key.as_ref(), multisig_pda.as_ref()];
    let (_pda, bump) = Pubkey::find_program_address(seeds, ctx.program_id);
    let signer_seeds: &[&[u8]] = &[b"patient_spl", case_id.as_bytes(), mint.key.as_ref(), multisig_pda.as_ref(), &[bump]];
    let signer = &[&signer_seeds[..]];

    // 1. Create the account at the PDA address
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(TOKEN_ACCOUNT_SIZE as usize);
    let create_account_ix = system_instruction::create_account(
        &payer.key(),
        &patient_spl_ata.key(),
        lamports,
        TOKEN_ACCOUNT_SIZE,
        &token_program.key(),
    );
    invoke_signed(
        &create_account_ix,
        &[
            payer.clone(),
            patient_spl_ata.clone(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer,
    )?;

    // 2. Initialize the token account
    let cpi_accounts = InitializeAccount {
        account: patient_spl_ata.clone(),
        mint: mint.clone(),
        authority: ctx.accounts.multisig_pda.clone(),
        rent: rent_sysvar,
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
    initialize_account(cpi_ctx)?;

    // Verify the seeds used for the patient_spl_ata account to ensure they match the expected values
    let expected_pda = Pubkey::create_program_address(&[b"patient_spl", case_id.as_bytes(), mint.key().as_ref(), multisig_pda.key().as_ref(), &[bump]], ctx.program_id)
        .map_err(|_| CuraChainError::ConstraintSeeds)?;
    require!(patient_spl_ata.key() == expected_pda, CuraChainError::ConstraintSeeds);

    Ok(())
}