
use solana_program::{
    account_info::{AccountInfo}, entrypoint::ProgramResult, msg,
};

use web3_utils::check::check_account_key;

use crate::{central_state, constants::return_vault_key, cpi::Cpi, state::NameStateRecordHeader, utils::transfer_by_chain::{transfer_by_referrer_chain}};

pub fn initialize_settle(
    accounts: &super::Accounts<'_, AccountInfo<'_>>,
    params: super::Params,
    name_state_data: &NameStateRecordHeader,
) -> ProgramResult {

    msg!("now the price: {:?}, and referrer all", name_state_data.highest_price);
    let (vault_key, _) = return_vault_key();
    check_account_key(accounts.vault, &vault_key)?;

    transfer_by_referrer_chain(
        &accounts, name_state_data.highest_price,
    )?;
    msg!("transfer profit and promote ok");
    
    let central_state_signer_seeds: &[&[u8]] = &[&crate::ID.to_bytes(), &[central_state::NONCE]];
    Cpi::transfer_name_account(
        accounts.naming_service_program, 
        accounts.central_state, 
        accounts.name, 
        accounts.root_domain, 
        accounts.new_domain_owner.key, 
        central_state_signer_seeds, 
        params.custom_price
    )?;
    
    Ok(())
}