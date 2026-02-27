use solana_program::{
     clock::Clock, msg, program_error::ProgramError, sysvar::Sysvar 
};

use crate::state::ReferrerRecordHeader;

#[cfg(not(feature = "devnet"))]
pub const TIME_LIMIT: i64 = 2592000; // 30 days in seconds
#[cfg(feature = "devnet")]
pub const TIME_LIMIT: i64 = 300; // 2 minutes in seconds

pub fn get_now_time() -> Result<i64, ProgramError> {
    let clock = Clock::get()?;
    Ok(clock.unix_timestamp)
}

/// Check if the given timestamp has exceeded TIME_LIMIT
/// Returns true if current time >= timestamp + TIME_LIMIT
pub fn can_settle(timestamp: i64) -> Result<bool, ProgramError> {
    let now = get_now_time()?;
    let expiration_time = timestamp.checked_add(TIME_LIMIT)
        .ok_or_else(|| {
            msg!("Timestamp overflow in time limit check");
            ProgramError::InvalidArgument
        })?;
    Ok(now >= expiration_time)
}

/// if the referrer was created within the last three days, it's not allowed to be a referrer
pub fn if_referrer_valid(
    referrer_state: ReferrerRecordHeader
) -> Result<bool, ProgramError> {
    let now = get_now_time()?;

    if now <= referrer_state.create_time + 1 {
        msg!("this account needs to wait for one day");
        return Ok(false);
    }

    Ok(true)
}