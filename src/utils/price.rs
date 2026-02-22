
use solana_program::{
   program_error::ProgramError
};


pub const ADVANCED_STORAGE: u64 = 50_000_000;
#[cfg(not(feature = "devnet"))]
pub const CREATE_ROOT_TARGET: u64 = 500000000000;
#[cfg(feature = "devnet")]
pub const CREATE_ROOT_TARGET: u64 = 200_000_000;


const RATE_DENOMINATOR: u128 = 1_000_000_000; // 1e9 精度
const MAX_RATE: u128 = 2 * RATE_DENOMINATOR; // 200%

pub fn share_with_cap(total: u64, rate: u64) -> Result<u64, ProgramError> {
    let rate = rate as u128;

    if rate > MAX_RATE {
        return Err(ProgramError::InvalidArgument);
    }

    let result = (total as u128)
        .checked_mul(rate)
        .and_then(|v| v.checked_div(RATE_DENOMINATOR))
        .ok_or(ProgramError::InvalidArgument)?;

    Ok(result as u64)
}

pub mod math {
    use solana_program::program_error::ProgramError;

    pub fn add(a: u64, b: u64) -> Result<u64, ProgramError> {
        a.checked_add(b).ok_or(ProgramError::InvalidArgument)
    }

    pub fn sub(a: u64, b: u64) -> Result<u64, ProgramError> {
        a.checked_sub(b).ok_or(ProgramError::InvalidArgument)
    }

    pub fn mul(a: u64, b: u64) -> Result<u64, ProgramError> {
        a.checked_mul(b).ok_or(ProgramError::InvalidArgument)
    }

    pub fn div(a: u64, b: u64) -> Result<u64, ProgramError> {
        a.checked_div(b).ok_or(ProgramError::InvalidArgument)
    }
}
