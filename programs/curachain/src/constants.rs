use anchor_lang::prelude::*;

#[constant]
pub const CASE_SEED: &[u8] = b"patient-case";
#[constant]
pub const ESCROW_SEED: &[u8] = b"escrow";
#[constant]
pub const VERIFIER_SEED: &[u8] = b"verifier";
#[constant]
pub const DONATION_SEED: &[u8] = b"donation";
#[constant]
pub const COUNTER_SEED: &[u8] = b"counter";
//#[constant]
//pub const ADMIN_PUBKEY: Pubkey = Pubkey::new_from_array([0; 32]);
#[constant]
pub const VERIFICATION_PERIOD: i64 = 3; 
#[constant]
pub const MIN_VERIFICATION_PARTICIPATION: f64 = 0.5;
#[constant]
pub const APPROVAL_THRESHOLD: f64 = 0.7;



