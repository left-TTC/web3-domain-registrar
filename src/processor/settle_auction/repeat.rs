
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::{invoke_signed}, program_error::ProgramError
};
use web3_domain_name_service::state::NameRecordHeader;
use web3_utils::check::check_account_key;

use crate::{central_state, constants::return_vault_key, cpi::Cpi, state::NameStateRecordHeader, utils::{promotion_inspect::add_domain_origin_owner_volume, share, transfer_by_chain::transfer_by_refferrer_chain}};
use solana_system_interface::instruction as system_instruction;


// Here we need to consider calls to the same address using different names.
pub fn repeat_settle(
    accounts: super::Accounts<'_, AccountInfo<'_>>,
    params: super::Params,
    name_account_data: NameRecordHeader,
    name_state_data: &NameStateRecordHeader,
) -> ProgramResult {

    if name_state_data.highest_price < name_account_data.custom_price{
        msg!("your price is too low");
        return Err(ProgramError::InvalidArgument);
    }

    check_account_key(accounts.origin_name_account_owner, &name_account_data.owner)?;

    let vault = accounts.vault;
    let (vault_key, _) = return_vault_key();
    check_account_key(vault, &vault_key)?;

    let domain_price = name_state_data.highest_price;
    msg!("transaction price: {:?}", domain_price);

    // invoke_signed(
    //     &system_instruction::transfer(
    //         vault.key,
    //         accounts.origin_name_account_owner.key,
    //         share(domain_price, 95)?,
    //     ),
    //     &[
    //         vault.clone(),
    //         accounts.origin_name_account_owner.clone(),
    //         accounts.system_program.clone(),
    //     ],
    //     &[vault_seeds], 
    // )?;
    let domain_price_lamports = share(domain_price, 52)?;
    **vault.try_borrow_mut_lamports()? -= domain_price_lamports;
    **accounts.refferrer_a.try_borrow_mut_lamports()? += domain_price_lamports;

    msg!("transfer to origin owner: {:?}", domain_price_lamports);
    msg!("transfer the domain fees to domain oringinal owner ok");

    transfer_by_refferrer_chain(
        &accounts, share(domain_price, 5)?, domain_price
    )?;
    msg!("transfer and promote ok");

    add_domain_origin_owner_volume(&accounts, domain_price)?;

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