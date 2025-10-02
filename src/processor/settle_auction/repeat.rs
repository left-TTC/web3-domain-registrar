
use solana_program::{
    account_info::{AccountInfo}, entrypoint::ProgramResult, msg, program::invoke, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent, sysvar::Sysvar 
};
use web3_domain_name_service::state::NameRecordHeader;
use web3_utils::check::check_account_key;

use crate::{central_state, cpi::Cpi, state::NameStateRecordHeader};
use solana_system_interface::instruction as system_instruction;

pub fn repeat_settle(
    accounts: super::Accounts<'_, AccountInfo<'_>>,
    params: super::Params,
    name_account_data: NameRecordHeader,
    name_state_data: NameStateRecordHeader,
    total_price: u64,
) -> ProgramResult {

    if name_state_data.highest_price < name_account_data.custom_price{
        msg!("your price is too low");
        return Err(ProgramError::InvalidArgument);
    }

    check_account_key(accounts.name_account_owner, &name_account_data.owner)?;

    invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, accounts.name_account_owner.key, total_price * 95 / 100
        ), &[
            accounts.fee_payer.clone(),
            accounts.name_account_owner.clone(),
            accounts.system_program.clone(),
        ]   
    )?;
    msg!("transfer the domain fees to domain oringinal owner ok");

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