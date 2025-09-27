
use solana_program::{
    account_info::AccountInfo, clock::Clock, program_error::ProgramError, sysvar::Sysvar
};
use web3_utils::pyth::get_domain_price_sol;


pub const ADVANCED_STORAGE: u64 = 5000000;
#[cfg(not(feature = "devnet"))]
pub const CREATE_ROOT_TARGET: u64 = 500000000000;
#[cfg(feature = "devnet")]
pub const CREATE_ROOT_TARGET: u64 = 20000000;

// $0.99
pub const START_PRICE: u64 = 1990000;

pub const DEPOSIT_RATIO: f64 = 0.1;

pub const PROJECT_START: u64 = 500000000;

//Get the required number of tokens
pub fn get_sol_price(
    sol_pyth_feed: &AccountInfo<'_>,
    usd_price: u64,
) -> Result<u64, ProgramError> {
    let clock = Clock::get()?;
    get_domain_price_sol(usd_price, sol_pyth_feed, &clock)
}
