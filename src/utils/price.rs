
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError,
};
use web3_utils::pyth::get_domain_price_sol;


pub const ADVANCED_STORAGE: u64 = 500000;
#[cfg(not(feature = "devnet"))]
pub const CREATE_ROOT_TARGET: u64 = 500000000000;
#[cfg(feature = "devnet")]
pub const CREATE_ROOT_TARGET: u64 = 20000000;

//Get the required number of tokens
pub fn get_sol_price(
    sol_pyth_feed: &AccountInfo<'_>,
    usd_price: u64,
) -> Result<u64, ProgramError> {
    get_domain_price_sol(usd_price, sol_pyth_feed)
}
