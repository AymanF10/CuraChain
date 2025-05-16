use anchor_lang::prelude::*;

use crate::states::{contexts::*, errors::*, events::*, constants::*};


/* There Is Gonna Be A Verifier Registry List
Where I Track All Added Verifiers To The System. 
This Will Be A PDA initialized One Time */

// Let's Initialize The Verifiers Registry, Multisig and Case Counter PDA accounts here
pub fn initialize_verifiers_list(ctx: Context<InitializeVerifiersRegistryMultisigAndCaseCounter>) -> Result<()> {
    let verifiers_registry = &mut ctx.accounts.verifiers_registry_list;
    let multisig = &mut ctx.accounts.multisig;

    let case_id_counter = &mut ctx.accounts.case_counter;
    case_id_counter.current_id = 0;
    case_id_counter.counter_bump = ctx.bumps.case_counter;


    verifiers_registry.all_verifiers = Vec::new();
    verifiers_registry.verifier_registry_bump = ctx.bumps.verifiers_registry_list;

    multisig.multisig_admin = ctx.accounts.admin.key();
    multisig.multisig_members = Vec::new();
    multisig.required_threshold = MULTISIG_THRESHOLD;
    multisig.multisig_bump = ctx.bumps.multisig;

    let message = format!("The Global Registry Of Verifiers and Multisig Has Been Initialized");
    emit!(GlobalRegistryInitializeEvent {
        message
    });

    Ok(())
}


pub fn add_verifier(ctx: Context<VerifierInfo>, verifier_address: Pubkey) -> Result<()> {
    // Let's get the context struct
    let verifier_info = &mut ctx.accounts.verifier;

    // Let's set the verifier status
    verifier_info.verifier_key = verifier_address;
    verifier_info.is_verifier = true;
    verifier_info.verifier_bump = ctx.bumps.verifier;

    // Let's Add This Verifier Address To The Global Verifiers Registry
    let verifiers_registry = &mut ctx.accounts.verifiers_list;

    // Add the verifier PDA account, and not just the address
    verifiers_registry.add_verifier_pda_to_list(verifier_info.key())?;
    let current_time = Clock::get()?.unix_timestamp;

    let message = format!("A Verifier With address, {} has been initialized to Global Registry of Verifiers At Time, {}", verifier_address, current_time);

    emit!(AddingNewVerifier {
        address: verifier_address,
        timestamp: current_time,
        message
    });

    Ok(())
}

pub fn remove_verifier(ctx: Context<VerifierInfo>, verifier_address: Pubkey) -> Result<()> {

    let verifier_info = &mut ctx.accounts.verifier;

    // Let's ensure the intended verifier to remove is the one we really want to remove
    require!(verifier_info.verifier_key == verifier_address, CuraChainError::InvalidVerifierAddress);

    // Remove Verifier PDA from the Global Registry
    let verifiers_registry = &mut ctx.accounts.verifiers_list;

    verifiers_registry.remove_verifier_pda_from_list(&verifier_info.key())?;

    // Let's set verifier status to false
    verifier_info.is_verifier = false;

    let current_time = Clock::get()?.unix_timestamp;
    let message = format!("The Verifier with address, {} has been disabled and no longer a verifier at Time, {}", verifier_address, current_time);
    emit!(RemovingExistingVerifier {
        address: verifier_address,
        timestamp: current_time,
        message
    });

    Ok(())
}


#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub enum VerifierOperationType {
    Add,
    Remove,
}