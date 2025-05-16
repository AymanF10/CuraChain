use anchor_lang::prelude::*;


use crate::states::contexts::*;



pub fn view_patient_case(ctx: Context<PatientDetails>, case_id: String) -> Result<()> {

    let patient_details = &ctx.accounts.patient_case;

    // Let's Log the patient case details below
    
    msg!("--------------------- PATIENT CASE DETAILS -------------------------");
    
    msg!("Specified Case ID is: {}", case_id);
    msg!("Patient Case Description: {}", patient_details.case_description);
    msg!("Encrypted Link To Patient Case Medical Records: {}", patient_details.link_to_records);
    msg!("Patient Case Verification Status: {}", patient_details.is_verified);

    msg!("Total Amount Needed For Case: {}", patient_details.total_amount_needed);
    msg!("Total Amount Raised So Far: {}", patient_details.total_sol_raised);
    msg!("Patient Case Funding Status: {}", patient_details.case_funded);

    Ok(())
}