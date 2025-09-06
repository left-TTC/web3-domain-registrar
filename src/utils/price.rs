
use solana_program::{
    account_info::AccountInfo, clock::Clock, program_error::ProgramError,
    sysvar::Sysvar, msg
};

use web3_name_service_utils::{checks::{check_account_key}, fp_math::fp32_div, tokens::SupportedToken};

use crate::{constants::{WSOL_MINT}};

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

    let token = SupportedToken::from_mint(&WSOL_MINT)?;
    check_account_key(sol_pyth_feed, &token.price_feed_account_key())?;
    
    let token_price = web3_name_service_utils::pyth::get_oracle_price_fp32_v2(
        &WSOL_MINT,
        sol_pyth_feed,
        token.decimals(),
        6,
        &Clock::get().unwrap(),

        //origin: 60
        6000,
    )?;

    let sol_price = fp32_div(usd_price, token_price).unwrap();

    Ok(sol_price)
}
