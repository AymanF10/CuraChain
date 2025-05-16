
use anchor_lang::prelude::*;

use crate::states::{contexts::*, errors::*, MultisigApprovals, MULTISIG_THRESHOLD};



// 1. First Add Or Remove Multisig

pub fn add_or_remove_members(ctx: Context<AddorRemoveMultisigMember>, member_addresses: Vec<Pubkey>, multisig_op_type: MultisigOperationType) -> Result<()> {

    require!(member_addresses.len() < 7, CuraChainError::TooManyMembers);
    let multisig = &mut ctx.accounts.multisig;
    
   match multisig_op_type {
    MultisigOperationType::AddMember => {
        // We Iterate Through The User defined member_addresses and add them up to multisig_members
        for signer in member_addresses.iter() {
            // Only Add If Signer Has Not Already Been Added
            if !multisig.multisig_members.contains(&signer) {
                multisig.multisig_members.push(*signer);
            } else {
                //return err!(CuraChainError::SignerAlreadyExists);
                // No Need To Error Otherwise Might Halt Iteration
                continue;
            }
        }
    },

    MultisigOperationType::RemoveMember => {
        for signer_to_remove in member_addresses.iter() {
            // Get Index of the signer to remove
            let index = multisig
                .multisig_members
                .iter()
                .position(|s| s == signer_to_remove)
                .ok_or(CuraChainError::NotMultisigMember)?;

            // Swap This Index With The Last Index In The Array
            let last = multisig.multisig_members.len() - 1;
            if index != last {
                multisig.multisig_members[index] = multisig.multisig_members[last];
            }

            // Pop The Last Element of multisig_members array off
            multisig.multisig_members.pop();
        }
    }
   }
   
    Ok(())
}



// A Multisig Member Can Propose Full or Partial Funds Release
pub fn propose_funds_release(ctx: Context<ProposeFundRelease>, case_id: String, proposal_index: u64) -> Result<()> {

    let patient_case = &ctx.accounts.patient_case;
    let proposal = &mut ctx.accounts.proposal;

    // Case Must Be Verified, And At Least A Non-Zero Donation Made To Either Escrow PDA or Patient Token Vault
    require!(patient_case.is_verified == true, CuraChainError::CaseNotYetVerified);

    let has_any_spl = patient_case
        .spl_donations
        .iter()
        .any(|donation| donation.total_mint_amount > 0);

    require!(patient_case.total_sol_raised > 0 || has_any_spl, CuraChainError::NoDonationsMade);

    // Fill Proposal
    proposal.case_id = case_id;
    proposal.proposal_index = proposal_index;
    proposal.voted_multisig.push( MultisigApprovals {
        multisig_member: ctx.accounts.proposer.key(),
        approval: true,
    });
    proposal.approved = false;
    proposal.proposal_bump = ctx.bumps.proposal;
    Ok(())
}


// Multisig Members Can Vote On A Proposal
pub fn proposal_approve(ctx: Context<ApproveProposal>, case_id: String, proposal_index: u64, approval: bool) -> Result<()> {

    let proposal = &mut ctx.accounts.proposal;

    // Ensure Input case_id and Proposal index is Correct
    require!(proposal.case_id == case_id, CuraChainError::NoProposalMade);
    require!(proposal.proposal_index == proposal_index, CuraChainError::InvalidProposalIndex);

    // Check That Multisig Member Can Only Vote Once
    let already_voted = proposal
        .voted_multisig
        .iter()
        .any(|v| v.multisig_member == ctx.accounts.multisig_member.key());

    require!(!already_voted, CuraChainError::MultisigMemberVoted);

    // Record Votes
    proposal.voted_multisig.push( MultisigApprovals {
        multisig_member: ctx.accounts.multisig_member.key(),
        approval,
    });

    // SET PROPOSAL APPROVAL TO TRUE IF MULTISIG THRESHOLD HAS ACCEPTED THE PROPOSAL
    let mut true_count = 0;

    for i in proposal.voted_multisig.iter() {
        if i.approval == true {
            true_count = true_count + 1;
        } else {
            continue;
        }
    }
    // Check If Approval threshold Has Reached
    if true_count >= MULTISIG_THRESHOLD {
        proposal.approved = true;
    }
    

    Ok(())
}




// Multisig Operations
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub enum MultisigOperationType {
    AddMember,

    RemoveMember,
}