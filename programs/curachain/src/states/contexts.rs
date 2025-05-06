use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface, TokenAccount};
use crate::states::{Administrator, CaseCounter, PatientCase, CaseIDLookup, DonorInfo, Verifier, VerifiersList};
use crate::states::errors::CuraChainError;

// THE ADMIN CONFIG STRUCT
#[derive(Accounts)]
#[instruction(admin_address: Pubkey)]
pub struct AdminConfig<'info> {
    #[account(
        init,
        payer = initializer,
        space = 8 + 32 + 1 + 1,
        seeds = [b"admin", admin_address.key().as_ref()],
        bump
    )]
    pub admin_account: Account<'info, Administrator>,

    #[account(mut)]
    pub initializer: Signer<'info>,

    pub system_program: Program<'info, System>,
}


//There should be only the administrator who can call this function to add the verifier badge to others
#[derive(Accounts)]
#[instruction(verifier_address: Pubkey)]
pub struct VerifierInfo<'info> {
    #[account(
        mut,
        constraint = admin.key() == admin_account.admin_pubkey.key() @ CuraChainError::OnlyAdmin,
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"admin", admin.key().as_ref()],
        bump = admin_account.bump
    )]
    pub admin_account: Account<'info, Administrator>,

    // let's create the Verifier PDA
    #[account(
        init_if_needed,
        payer = admin,
        space = 8 + 32 + 1 + 1,
        seeds = [b"verifier_role", verifier_address.key().as_ref()],
        bump,
    )]
    pub verifier: Account<'info, Verifier>,

    // Adding the Global Verifiers List PDA here
    #[account(
        mut,
        seeds = [b"verifiers_list"],
        bump = verifiers_list.verifier_registry_bump,
    )]
    pub verifiers_list: Account<'info, VerifiersList>,

    pub system_program: Program<'info, System>,
}





// Context Struct For Initializing The Global Verifiers Registry PDA account

#[derive(Accounts)]
pub struct InitializeVerifiersRegistryAndCaseCounter<'info> {
    #[account(
        mut,
        constraint = admin.key() == admin_account.admin_pubkey.key() @ CuraChainError::OnlyAdmin,
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"admin", admin.key().as_ref()],
        bump = admin_account.bump
    )]
    pub admin_account: Account<'info, Administrator>,

    #[account(
        init,
        payer = admin,
        seeds = [b"verifiers_list"],
        bump,
        space = 8 + 4 + (32 * 100) + 1,
    )]
    pub verifiers_registry_list: Account<'info, VerifiersList>,

    // Case Counter PDA here
    #[account(
        init,
        payer = admin,
        seeds = [b"case_counter"],
        bump,
        space = 8 + 8 + 1,
    )]
    pub case_counter: Account<'info, CaseCounter>,

    pub system_program: Program<'info, System>,
}

// INITIALIZE PATIENT CASE context
#[derive(Accounts)]
pub struct InitializePatientCase<'info> {
    // Signer is patient
    #[account(mut)]
    pub patient: Signer<'info>,

    #[account(
        init,
        payer = patient,
        space = 8 + PatientCase::INIT_SPACE,
        seeds = [b"patient", patient.key().as_ref()],
        bump
    )]
    pub patient_case: Account<'info, PatientCase>,

    // let's bring the Case Counter PDA here
    #[account(
        mut,
        seeds = [b"case_counter"],
        bump = case_counter.counter_bump,
    )]
    pub case_counter: Account<'info, CaseCounter>,

    // Let's Bring Up The Case ID Lookup PDA here
    #[account(
        init,
        payer = patient,
        space = 8 + CaseIDLookup::INIT_SPACE,
        seeds = [b"case_lookup",
        format!("CASE{:04}", case_counter.current_id + 1).as_bytes()],
        bump
    )]
    pub case_lookup: Account<'info, CaseIDLookup>,

    pub system_program: Program<'info, System>,
}



// A VIEW INSTRUCTION FOR TRACKING PATIENT CASE STATUS ON-CHAIN
#[derive(Accounts)]
#[instruction(case_id: String)]
pub struct PatientDetails<'info> {
    #[account(mut)]
    pub caller: Signer<'info>,

    // Let's get the Case Lookup PDA using the specified case ID of the original format, CASE####
    #[account(
        seeds = [b"case_lookup", case_id.as_bytes()],
        bump = case_lookup.case_lookup_bump,
        constraint = case_lookup.case_id_in_lookup == case_id @CuraChainError::InvalidCaseID,
    )]
    pub case_lookup: Account<'info, CaseIDLookup>,

    #[account(
        seeds = [b"patient", case_lookup.patient_address.as_ref()],
        bump = patient_case.patient_case_bump,
        constraint = patient_case.key() == case_lookup.patient_pda.key() @ CuraChainError::InvalidCaseID,
        constraint = patient_case.case_id == case_id @ CuraChainError::InvalidCaseID,
    )]
    pub patient_case: Account<'info, PatientCase>,

}

// INITIALIZE THE VERIFICATION INSTRUCTION
#[derive(Accounts)]
#[instruction(case_id: String)]
pub struct VerifyPatientCase<'info> {
    #[account(
        mut,
        constraint = verifier.key() == verifier_account.verifier_key.key() @ CuraChainError::OnlyVerifier,
    )]
    pub verifier: Signer<'info>,

    #[account(
        mut,
        seeds = [b"verifier_role", verifier.key().as_ref()],
        bump = verifier_account.verifier_bump
    )]
    pub verifier_account: Account<'info, Verifier>,

    // I think i should add the global verifiers registry so that i can query it for the total votes cast
    #[account(
        mut,
        seeds = [b"verifiers_list"],
        bump = verifiers_list.verifier_registry_bump,
    )]
    pub verifiers_list: Account<'info, VerifiersList>,

    // Let's get the Case Lookup PDA using the specified case ID of the original format, CASE####
    #[account(
        mut,
        seeds = [b"case_lookup", case_id.as_bytes()],
        bump = case_lookup.case_lookup_bump,
        constraint = case_lookup.case_id_in_lookup == case_id @CuraChainError::InvalidCaseID,
    )]
    pub case_lookup: Account<'info, CaseIDLookup>,

    #[account(
        mut,
        seeds = [b"patient", case_lookup.patient_address.as_ref()],
        bump = patient_case.patient_case_bump,
        constraint = patient_case.key() == case_lookup.patient_pda.key() @ CuraChainError::InvalidCaseID,
        constraint = patient_case.case_id == case_id @ CuraChainError::InvalidCaseID,
    )]
    pub patient_case: Account<'info, PatientCase>,

    /// CHECKED: This account does not exist yet, and may be created upon successful verification
    #[account(
        mut,
    )]
    pub patient_escrow: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}


// IF CASE FAILS VERIFICATION, WE CALL THIS INSTRUCTION TO CLOSE THE PATIENT CASE PDA
#[derive(Accounts)]
#[instruction(case_id: String)]

pub struct ClosePatientCase<'info> {

    // Anybody can call this instruction to close the patient case
    #[account(mut)]
    pub user: Signer<'info>,

    // Let's get the Case Lookup PDA using the specified case ID of the original format, CASE####
    #[account(
        mut,
        seeds = [b"case_lookup", case_id.as_bytes()],
        bump = case_lookup.case_lookup_bump,
        constraint = case_lookup.case_id_in_lookup == case_id @CuraChainError::InvalidCaseID,
    )]
    pub case_lookup: Account<'info, CaseIDLookup>,

    #[account(
        mut,
        close = user,// I would like the lamports to return to the person closing this account.
        seeds = [b"patient", case_lookup.patient_address.as_ref()],
        bump = patient_case.patient_case_bump,
        constraint = patient_case.key() == case_lookup.patient_pda.key() @ CuraChainError::InvalidCaseID,
        constraint = patient_case.case_id == case_id @ CuraChainError::InvalidCaseID,
    )]
    pub patient_case: Account<'info, PatientCase>,

    // Have The Verifier Registry So I Can Query The Expected Number Of Verifiers To Have Voted
    #[account(
        mut,
        seeds = [b"verifiers_list"],
        bump = verifiers_list.verifier_registry_bump,
    )]
    pub verifiers_list: Account<'info, VerifiersList>,

    pub system_program: Program<'info, System>,
}


// DONOR'S CONTEXT STRUCT
#[derive(Accounts)]
#[instruction(case_id: String, mint: Pubkey)]
pub struct Donation<'info> {
    #[account(mut)]
    pub donor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"case_lookup", case_id.as_bytes()],
        bump = case_lookup.case_lookup_bump,
        constraint = case_lookup.case_id_in_lookup == case_id @CuraChainError::InvalidCaseID,
    )]
    pub case_lookup: Account<'info, CaseIDLookup>,

    #[account(
        mut,
        seeds = [b"patient", case_lookup.patient_address.as_ref()],
        bump = patient_case.patient_case_bump,
        constraint = patient_case.key() == case_lookup.patient_pda.key() @ CuraChainError::InvalidCaseID,
        constraint = patient_case.case_id == case_id @ CuraChainError::InvalidCaseID,
    )]
    pub patient_case: Account<'info, PatientCase>,

    /// CHECK: This account has already been created and is safe because the PDA is derived and checked in the instruction.
    #[account(mut, owner = system_program.key())]
    pub patient_escrow: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = donor,
        seeds = [b"donor", donor.key().as_ref()],
        bump,
        space = 8 + DonorInfo::INIT_SPACE,
    )]
    pub donor_account: Account<'info, DonorInfo>,

    // NEW: SPL Token Support 
    #[account(mut)]
    pub donor_ata: Option<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub patient_ata: Option<InterfaceAccount<'info, TokenAccount>>,

    /// The token interface program
    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,

    /// CHECK: This is the multisig PDA (authority/owner of the SPL token account)
    #[account(mut)]
    pub multisig_pda: AccountInfo<'info>,

    /// The mint for SPL token donations
    pub mint: Option<AccountInfo<'info>>,
}

#[derive(Accounts)]
#[instruction(case_id: String)]
pub struct ReleaseFunds<'info> {
    #[account(
        mut,
        seeds = [b"case_lookup", case_id.as_bytes()],
        bump = case_lookup.case_lookup_bump,
        constraint = case_lookup.case_id_in_lookup == case_id @CuraChainError::InvalidCaseID,
    )]
    pub case_lookup: Account<'info, CaseIDLookup>,

    #[account(
        mut,
        seeds = [b"patient", case_lookup.patient_address.as_ref()],
        bump = patient_case.patient_case_bump,
        constraint = patient_case.key() == case_lookup.patient_pda.key() @ CuraChainError::InvalidCaseID,
    )]
    pub patient_case: Account<'info, PatientCase>,

    /// CHECK: This account has already been created and is safe because the PDA is derived and checked in the instruction.
    #[account(mut)]
    pub patient_escrow: AccountInfo<'info>,

    /// CHECK: This is the treatment facility's address and is validated in the instruction logic.
    #[account(mut)]
    pub facility_address: AccountInfo<'info>,

    #[account(mut, constraint = admin.key() == admin_account.admin_pubkey.key() @ CuraChainError::OnlyAdmin)]
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"admin", admin.key().as_ref()], bump = admin_account.bump)]
    pub admin_account: Account<'info, Administrator>,

    #[account(mut)]
    pub verifier1: Signer<'info>,
    #[account(mut)]
    pub verifier2: Signer<'info>,
    #[account(mut)]
    pub verifier3: Signer<'info>,

    #[account(
        seeds = [b"verifier_role", verifier1.key().as_ref()],
        bump = verifier1_pda.verifier_bump,
        constraint = verifier1_pda.verifier_key == verifier1.key() @ CuraChainError::InvalidVerifier
    )]
    pub verifier1_pda: Account<'info, Verifier>,

    #[account(
        seeds = [b"verifier_role", verifier2.key().as_ref()],
        bump = verifier2_pda.verifier_bump,
        constraint = verifier2_pda.verifier_key == verifier2.key() @ CuraChainError::InvalidVerifier
    )]
    pub verifier2_pda: Account<'info, Verifier>,

    #[account(
        seeds = [b"verifier_role", verifier3.key().as_ref()],
        bump = verifier3_pda.verifier_bump,
        constraint = verifier3_pda.verifier_key == verifier3.key() @ CuraChainError::InvalidVerifier
    )]
    pub verifier3_pda: Account<'info, Verifier>,

    #[account(
        mut,
        seeds = [b"verifiers_list"],
        bump = verifiers_list.verifier_registry_bump
    )]
    pub verifiers_list: Account<'info, VerifiersList>,

    /// CHECK: This is the multisig PDA (authority/owner of the SPL token accounts)
    #[account(mut)]
    pub multisig_pda: AccountInfo<'info>,

    /// The token interface program
    pub token_program: Interface<'info, TokenInterface>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(case_id: String)]
pub struct AdminOverrideCase<'info> {
    #[account(
        mut,
        constraint = admin.key() == admin_account.admin_pubkey.key() @ CuraChainError::OnlyAdmin,
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"admin", admin.key().as_ref()],
        bump = admin_account.bump
    )]
    pub admin_account: Account<'info, Administrator>,

    #[account(
        mut,
        seeds = [b"case_lookup", case_id.as_bytes()],
        bump = case_lookup.case_lookup_bump,
        constraint = case_lookup.case_id_in_lookup == case_id @CuraChainError::InvalidCaseID,
    )]
    pub case_lookup: Account<'info, CaseIDLookup>,

    #[account(
        mut,
        seeds = [b"patient", case_lookup.patient_address.as_ref()],
        bump = patient_case.patient_case_bump,
        constraint = patient_case.key() == case_lookup.patient_pda.key() @ CuraChainError::InvalidCaseID,
        constraint = patient_case.case_id == case_id @ CuraChainError::InvalidCaseID,
    )]
    pub patient_case: Account<'info, PatientCase>,

    /// CHECK: This account does not exist yet and will be created as the escrow PDA for the patient case. It is safe because the PDA is derived and checked in the instruction.
    #[account(
        mut,
        // This account does not exist yet, and may be created upon admin override verification
    )]
    pub patient_escrow: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}