
use solana_program::{
   program_error::ProgramError
};


pub const ADVANCED_STORAGE: u64 = 50_000_000;
#[cfg(not(feature = "devnet"))]
pub const CREATE_ROOT_TARGET: u64 = 500000000000;
#[cfg(feature = "devnet")]
pub const CREATE_ROOT_TARGET: u64 = 200_000_000;


pub fn share(total: u64, percent: u64) -> Result<u64, ProgramError> {
    total.checked_mul(percent)
         .and_then(|v| v.checked_div(100))
         .ok_or(ProgramError::InvalidArgument)
}