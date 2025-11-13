
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg,  program_error::ProgramError
};
use web3_domain_name_service::state::NameRecordHeader;
use web3_utils::check::check_account_key;
use solana_program::program_pack::Pack;
use crate::{central_state, constants::return_vault_key, cpi::Cpi, state::{NameStateRecordHeader, get_referrer_record_key, ReferrerRecordHeader}, utils::{share, transfer_by_chain::transfer_by_referrer_chain}};


// Here we need to consider calls to the same address using different names.
pub fn repeat_settle(
    accounts: super::Accounts<'_, AccountInfo<'_>>,
    params: super::Params,
    name_account_data: NameRecordHeader,
    name_state_data: &NameStateRecordHeader,
) -> ProgramResult {

    check_account_key(accounts.origin_name_account_owner, &name_account_data.owner)?;

    let vault = accounts.vault;
    let (vault_key, _) = return_vault_key();
    check_account_key(vault, &vault_key)?;

    let domain_price = name_state_data.highest_price;
    msg!("transaction price: {:?}", domain_price);

    transfer_by_referrer_chain(
        &accounts, share(domain_price, 5)?
    )?;
    msg!("add referrer profit and performance and up level ok");

    let origin_owner = accounts.origin_name_account_owner;
    let origin_owner_referrer_record = accounts.origin_name_owner_record;

    let (origin_owner_referrer_record_key, _) = get_referrer_record_key(origin_owner.key);
    check_account_key(origin_owner_referrer_record, &origin_owner_referrer_record_key)?;
   
    let mut data_origin_ref = origin_owner_referrer_record.try_borrow_mut_data()?;
    let mut origin_owner_record_data = 
        ReferrerRecordHeader::unpack_from_slice(
            &data_origin_ref
        )?;

    let get_lamports = share(domain_price, 95)?;
    
    // the domain origin owner's account will only add profit
    origin_owner_record_data.profit =
        origin_owner_record_data.profit
        .checked_add(get_lamports)
        .ok_or(ProgramError::InvalidArgument)?;

    origin_owner_record_data.pack_into_slice(&mut *data_origin_ref);
    msg!("add origin owner only profit ok: {:?}", get_lamports);

    let central_state_signer_seeds: &[&[u8]] = &[&crate::ID.to_bytes(), &[central_state::NONCE]];
    Cpi::transfer_name_account(
        accounts.naming_service_program, 
        accounts.central_state, 
        accounts.name, 
        accounts.root_domain, 
        accounts.fee_payer.key, 
        central_state_signer_seeds,
        params.custom_price
    )?;

    Ok(())
}