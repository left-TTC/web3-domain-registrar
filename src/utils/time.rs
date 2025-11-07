use solana_program::{
     clock::Clock, program_error::ProgramError, sysvar::Sysvar 
};


#[cfg(not(feature = "devnet"))]
pub const AUCTION_TIME_LIMIT: i64 = 259200;
#[cfg(feature = "devnet")]
pub const AUCTION_TIME_LIMIT: i64 = 600; // 10min

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

    let auction_expiration_time = name_state_update_time + AUCTION_TIME_LIMIT;

    if auction_expiration_time > now_timestamp {
        return Ok(TIME::AUCTION)
    }

    Ok(TIME::PENDING)
}

pub fn get_now_time() -> Result<i64, ProgramError> {
    let clock = Clock::get()?;
    Ok(clock.unix_timestamp)
}