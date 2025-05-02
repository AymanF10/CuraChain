use anchor_lang::prelude::*;

use crate::states::{contexts::*, events::*};


/*
1. The Case Should Only Be Closed If At Least 50% of Verifiers Have Voted, and Yet Still
the 70% approval threshold was not Agreed Upon.

NOTE BETTER::: We will implement the above checks as an access_control, not as anchor constraints, as 
we don't want the checks to be done on the account level, but prior to even account-level validations*/

pub fn close_rejected_case(ctx: Context<ClosePatientCase>, case_id: String) -> Result<()> {
    

    let patient_case = &mut ctx.accounts.patient_case;

    let user_closing = ctx.accounts.user.to_account_info();

    // Let's Call The Solana Close instruction straight away, and no need to cpi into it.
    patient_case.close(user_closing)?;


    // CATCHING THIS EVENT ON-CHAIN ANYTIME A REJECTED CASE IS CLOSED
    let current_time = Clock::get()?.unix_timestamp;
    let message = format!("Patient Case with ID, {} Was Rejected And Is Therefore Being Closed At Time, {}", case_id, current_time);

    emit!(
        CloseRejectedPatientCase {
            message,
            case_id,
            timestamp: current_time,
        }
    );

    

    Ok(())
}