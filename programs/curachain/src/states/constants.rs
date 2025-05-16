use anchor_lang::prelude::*;


pub const SCALE: u32 = 10000;

// Wrapped SOL Mint Address
//pub const NATIVE_SOL_MINT_ADDRESS: Pubkey = spl_token::native_mint::id();
pub const NATIVE_SOL_ADDRESS: Pubkey = Pubkey::new_from_array([0u8;32]);
pub const MULTISIG_THRESHOLD: u8 = 3;
pub const DONATION_BUFFER: u64 = 1_000_000_000 * (10_u64.pow(6));

pub const ALLOWED_VERIFICATION_TIME: u64 =  864_000;