
use solana_program::{
    account_info::{AccountInfo}, entrypoint::ProgramResult, msg, program_pack::Pack, rent::Rent, sysvar::Sysvar 
};

use web3_domain_name_service::{state::NameRecordHeader};
use web3_utils::check::check_account_key;

use crate::{central_state, constants::return_vault_key, cpi::Cpi, state::{NameStateRecordHeader}, utils::{transfer_by_chain}};

pub fn initialize_settle(
    accounts: super::Accounts<'_, AccountInfo<'_>>,
    params: super::Params,
    name_state_data: &NameStateRecordHeader,
    hased_name: Vec<u8>,
    hashed_reverse_lookup: Vec<u8>,
) -> ProgramResult {

    msg!("now the price: {:?}, and refferrer all", name_state_data.highest_price);
    let (vault_key, _) = return_vault_key();
    check_account_key(accounts.vault, &vault_key)?;

    transfer_by_chain::transfer_by_refferrer_chain(
        &accounts, name_state_data.highest_price, name_state_data.highest_price
    )?;
    msg!("transfer and promote ok");
    
    let rent_sysvar = accounts.rent_sysvar;
    let rent = Rent::from_account_info(accounts.rent_sysvar)?;
    
    let central_state_signer_seeds: &[&[u8]] = &[&crate::ID.to_bytes(), &[central_state::NONCE]];
    Cpi::create_name_account(
        accounts.naming_service_program, 
        accounts.system_program, 
        accounts.name, 
        accounts.fee_payer, 
        accounts.root_domain, 
        accounts.central_state,
        hased_name,
        rent.minimum_balance(NameRecordHeader::LEN as usize),
        central_state_signer_seeds,
        params.custom_price,
    )?;

    if accounts.reverse_lookup.data_len() == 0 {
        Cpi::create_reverse_lookup_account(
            accounts.naming_service_program, 
            accounts.system_program, 
            accounts.reverse_lookup, 
            accounts.fee_payer, 
            params.domain_name, 
            hashed_reverse_lookup, 
            accounts.central_state, 
            rent_sysvar, 
            central_state_signer_seeds, 
            None, 
            None
        )?;
    }
    
    Ok(())
}