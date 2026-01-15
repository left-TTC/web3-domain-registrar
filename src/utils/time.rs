use solana_program::{
     clock::Clock, msg, program_error::ProgramError, sysvar::Sysvar 
};

use crate::state::ReferrerRecordHeader;


#[cfg(not(feature = "devnet"))]
pub const AUCTION_TIME_LIMIT: i64 = 259200;
#[cfg(feature = "devnet")]
pub const AUCTION_TIME_LIMIT: i64 = 120; // 10min

#[derive(PartialEq)]
pub enum TIME {
    ERROR,
    AUCTION,
    PENDING
}

pub fn check_state_time(
    name_state_update_time: i64,
) -> Result<TIME, ProgramError> {
    let now_timestamp = get_now_time()?;

    if name_state_update_time > now_timestamp {
        return Ok(TIME::ERROR)
    }

    let auction_expiration_time = name_state_update_time
        .checked_add(AUCTION_TIME_LIMIT)
        .ok_or(ProgramError::InvalidArgument)?;

    if auction_expiration_time > now_timestamp {
        return Ok(TIME::AUCTION)
    }

    Ok(TIME::PENDING)
}

pub fn get_now_time() -> Result<i64, ProgramError> {
    let clock = Clock::get()?;
    Ok(clock.unix_timestamp)
}


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