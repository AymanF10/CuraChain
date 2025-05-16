use std::u8;

use anchor_lang::prelude::*;

use crate::states::errors::*;


// CREATE THE ADMINISTRATOR ACCOUNT HERE
#[account]
pub struct Administrator {
    pub admin_pubkey: Pubkey,

    pub is_active: bool,

    pub bump: u8,
}


// CREATE THE MULTISIG ACCOUNT HERE
#[account]
#[derive(InitSpace)]
pub struct Multisig {
    pub multisig_admin: Pubkey,

    #[max_len(5)]
    pub multisig_members: Vec<Pubkey>,

    pub required_threshold: u8,

    pub multisig_bump: u8
}


// CREATE A CASE COUNTER PDA THAT WILL INCREMENT AND ASSIGN EACH CASE AN ID
// OF THE FORMAT, CASE + (RANDOM 4 NUMBER)
#[account]
pub struct CaseCounter {
    pub current_id: u64,
    pub counter_bump: u8,
}

// CREATE THE PATIENT ACCOUNT HERE
#[account]
#[derive(InitSpace)]
pub struct PatientCase {
    pub patient_pubkey: Pubkey,

    #[max_len(50)]
    pub case_description: String,

    pub total_amount_needed: u64,

    pub total_sol_raised: u64,

    #[max_len(20)]
    pub spl_donations: Vec<SplDonations>,

    #[max_len(10)]
    pub case_id: String,
    
    pub verification_yes_votes:u8,
    // list of voted verifiers on a case
    #[max_len(25)]
    pub voted_verifiers: Vec<Pubkey>,

    pub verification_no_votes: u8,
    
    pub is_verified: bool,

    pub patient_case_bump: u8,

    pub case_funded: bool,

    pub submission_time: i64,

    #[max_len(64)]
    pub link_to_records: String,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, InitSpace, PartialEq, Copy)]
pub struct SplDonations {
    pub mint: Pubkey,

    pub total_mint_amount: u64,

    pub patient_token_vault: Pubkey,
}


// CASE ID LOOKUP
#[account]
#[derive(InitSpace)]
pub struct CaseIDLookup{
    #[max_len(10)]
    pub case_id_in_lookup: String,

    pub patient_pda: Pubkey,

    pub patient_address: Pubkey,

    pub case_lookup_bump: u8,

    pub patient_escrow_bump: u8,
}



// CREATE THE VERIFIER INFO CONFIG HERE
#[account]
pub struct Verifier{
    pub verifier_key: Pubkey,
    pub is_verifier: bool,
    pub verifier_bump: u8,
}


// CREATE A VERIFIER REGISTRY LIST TO STORE ALL VERIFIERS' PDA accounts here
#[account]
pub struct VerifiersList {
    pub all_verifiers: Vec<Pubkey>,
    pub verifier_registry_bump: u8,
}

impl VerifiersList {
    // Function to Add verifier Onto The Verifiers List
    pub fn add_verifier_pda_to_list(&mut self, verifier_to_add: Pubkey) -> Result<()> {
        require!(!self.all_verifiers.contains(&verifier_to_add), CuraChainError::VerifierAlreadyExists);

        self.all_verifiers.push(verifier_to_add);
        Ok(())
    }

    // Function to Remove Verifier From The Verifiers List
    pub fn remove_verifier_pda_from_list(&mut self, verifier_to_remove: &Pubkey) -> Result<()> {
        //require!(self.all_verifiers.contains(&verifier_to_remove), MedifundError::VerifierNotFound);

        if let Some(index) = self.all_verifiers.iter().position(|x| x == verifier_to_remove) {
            self.all_verifiers.remove(index);
            Ok(())
        } else {
            err!(CuraChainError::VerifierNotFound)
        }
    }
}


// CREATE A TRANSFER PROPOSAL HERE
#[account]
#[derive(InitSpace)]
pub struct Proposal {
    #[max_len(10)]
    pub case_id: String,

    pub proposal_index: u64,

    #[max_len(5)]
    pub voted_multisig: Vec<MultisigApprovals>,

    pub approved: bool,

    pub executed: bool,

    pub proposal_bump: u8,
}


#[derive(AnchorDeserialize, AnchorSerialize, InitSpace, Clone, Copy)]
pub struct MultisigApprovals {
    pub multisig_member: Pubkey,

    pub approval: bool,
}
// CREATE A DONOR INFO PDA HERE
#[account]
#[derive(InitSpace)]
pub struct DonorInfo {
    pub donor_address: Pubkey,

    pub donor_bump: u8,

    pub total_donations: u64,

    #[max_len(1000)]
    pub donated_cases: Vec<[u8;8]>,
}
   