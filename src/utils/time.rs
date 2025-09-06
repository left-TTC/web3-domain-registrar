use solana_program::{
     clock::Clock, program_error::ProgramError,
    sysvar::Sysvar, 
};


#[cfg(not(feature = "devnet"))]
pub const TIME_LIMIT: i64 = 259200;
#[cfg(feature = "devnet")]
pub const TIME_LIMIT: i64 = 60;

pub fn check_state_time_valid(
    name_state_account_time: i64,
) -> Result<bool, ProgramError> {

    let clock = Clock::get()?;
    let now_timestamp = clock.unix_timestamp;

    // example: a update on 9.6; 
    // and limitation is 3 days, it should settle at 9.9
    // now is 9.8, 9.6 add 3 days 
    if name_state_account_time + TIME_LIMIT > now_timestamp {
        Ok(true)
    }else {
        Ok(false)
    }
}

pub fn get_now_time() -> Result<i64, ProgramError> {
    let clock = Clock::get()?;
    Ok(clock.unix_timestamp)
}