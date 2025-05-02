use anchor_lang::prelude::*;
use crate::states::accounts::*;
use crate::states::errors::*;


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
#[instruction(case_id: String)]
pub struct Donation<'info> {
    #[account(mut)]
    pub donor: Signer<'info>,

    // Get Case Lookup pda using specified Case ID
    #[account(
        mut,
        seeds = [b"case_lookup", case_id.as_bytes()],
        bump = case_lookup.case_lookup_bump,
        constraint = case_lookup.case_id_in_lookup == case_id @CuraChainError::InvalidCaseID,
    )]
    pub case_lookup: Account<'info, CaseIDLookup>,

    // We Use the case_lookup to find the Patient case
    #[account(
        mut,
        seeds = [b"patient", case_lookup.patient_address.as_ref()],
        bump = patient_case.patient_case_bump,
        constraint = patient_case.key() == case_lookup.patient_pda.key() @ CuraChainError::InvalidCaseID,
        constraint = patient_case.case_id == case_id @ CuraChainError::InvalidCaseID,
    )]
    pub patient_case: Account<'info, PatientCase>,

    /// CHECKED: This account has already been created and it's safe now. 
    #[account(
        mut,
    )]
    pub patient_escrow: AccountInfo<'info>,

    // Donor Info PDA here
    #[account(
        init_if_needed,
        payer = donor,
        seeds = [b"donor", donor.key().as_ref()],
        bump,
        space = 8 + DonorInfo::INIT_SPACE,
    )]
    pub donor_account: Account<'info, DonorInfo>,

    pub system_program: Program<'info, System>,
}



// FUND RELEASE TO TREATMENT FACILITY
#[derive(Accounts)]
#[instruction(case_id: String)]
pub struct ReleaseFunds<'info> {
    // Get Case Lookup pda using specified Case ID
    #[account(
        mut,
        seeds = [b"case_lookup", case_id.as_bytes()],
        bump = case_lookup.case_lookup_bump,
        constraint = case_lookup.case_id_in_lookup == case_id @CuraChainError::InvalidCaseID,
    )]
    pub case_lookup: Account<'info, CaseIDLookup>,

    // We Use the case_lookup to find the Patient case
    #[account(
        mut,
        seeds = [b"patient", case_lookup.patient_address.as_ref()],
        bump = patient_case.patient_case_bump,
        constraint = patient_case.key() == case_lookup.patient_pda.key() @ CuraChainError::InvalidCaseID,
        constraint = patient_case.case_id == case_id @ CuraChainError::InvalidCaseID,
    )]
    pub patient_case: Account<'info, PatientCase>,

    /// CHECKED: This account has already been created and it's safe now. 
    #[account(
        mut,
        //seeds = [b"patient_escrow", case_id.as_bytes().as_ref() ,patient_case.key().as_ref(),],
        //bump = case_lookup.patient_escrow_bump,
        owner = system_program.key(),
    )]
    pub patient_escrow: AccountInfo<'info>,

    ///CHECKED: The Facility Address To Receive Funds For Patient Treatment
    #[account(mut)]
    pub facility_address: AccountInfo<'info>,

    // The Multisig To Sign The Transactions, 1 Admins, plus 3 Verifiers
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


    // The Verifiers For The Mutlisig
    
    #[account(mut)]
    pub verifier1: Signer<'info>,

    #[account(mut)]
    pub verifier2: Signer<'info>,

    #[account(mut)]
    pub verifier3: Signer<'info>,

    
    // Get Verifiers PDAs
    #[account(
        mut,
        seeds = [b"verifier_role", verifier1.key().as_ref()],
        bump = verifier1_pda.verifier_bump
    )]
    pub verifier1_pda: Account<'info, Verifier>,

    #[account(
        mut,
        seeds = [b"verifier_role", verifier2.key().as_ref()],
        bump = verifier2_pda.verifier_bump
    )]
    pub verifier2_pda: Account<'info, Verifier>,

    #[account(
        mut,
        seeds = [b"verifier_role", verifier3.key().as_ref()],
        bump = verifier3_pda.verifier_bump
    )]
    pub verifier3_pda: Account<'info, Verifier>,

    // Verifiers Registry To Confirm the verifiers
    #[account(
        seeds = [b"verifiers_list"],
        bump = verifiers_list.verifier_registry_bump,
    )]
    pub verifiers_list: Account<'info, VerifiersList>,

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
        constraint = case_lookup.case_id_in_lookup == case_id @ CuraChainError::InvalidCaseID,
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