
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::{invoke_signed}, program_error::ProgramError, program_pack::Pack
};

use solana_system_interface::instruction as system_instruction;
use web3_utils::check::check_account_key;

use crate::{central_state, constants::return_vault_key, state::{RefferrerRecordHeader, get_refferrer_record_key}, utils::{get_hashed_name, promotion_inspect::promotion_inspect, share}};

pub fn transfer_by_refferrer_chain(
    accounts: &crate::processor::settle_auction::Accounts<'_, AccountInfo<'_>>,
    refferrer_lamports: u64,
    domain_price_lamports: u64,
) -> ProgramResult {

    let vault = accounts.vault;
    

    let domain_owner = accounts.new_domain_owner;
    let (owner_record, _) = get_refferrer_record_key(domain_owner.key);
    check_account_key(accounts.refferrer_record, &owner_record)?;

    let refferrer_usr_data = 
        RefferrerRecordHeader::unpack_from_slice(&accounts.refferrer_record.data.borrow())?;

    let mut who_vault: u8 = 0;

    if &refferrer_usr_data.refferrer_account != vault.key{

        if &refferrer_usr_data.refferrer_account != accounts.refferrer_a.key {
            msg!("provide fault refferrer A");
            return Err(ProgramError::InvalidArgument);
        }

        if let Some(refferrer_a_record) = accounts.refferrer_a_record {

            let (verify_record_a, _) = get_refferrer_record_key(&accounts.refferrer_a.key);
            check_account_key(refferrer_a_record, &verify_record_a)?;

            let a_record_data = 
                RefferrerRecordHeader::unpack_from_slice(&refferrer_a_record.data.borrow())?;

            if &a_record_data.refferrer_account != vault.key {
                
                if let Some(refferrer_b) = accounts.refferrer_b {
                    if &a_record_data.refferrer_account != refferrer_b.key {
                        msg!("provide fault refferrer B");
                        return Err(ProgramError::InvalidArgument);
                    }

                    if let Some(refferrer_b_record) = accounts.refferrer_b_record {

                        let (verify_record_b, _) = get_refferrer_record_key(&refferrer_b.key);
                        check_account_key(refferrer_b_record, &verify_record_b)?;

                        let b_record_data = 
                            RefferrerRecordHeader::unpack_from_slice(&refferrer_b_record.data.borrow())?;

                        if &b_record_data.refferrer_account != vault.key {

                            if let Some(refferrer_c) = accounts.refferrer_c {

                                if &b_record_data.refferrer_account != refferrer_c.key {
                                    msg!("provide fault refferrer C");
                                    return Err(ProgramError::InvalidArgument);
                                }

                                if let Some(refferrer_c_record)  = accounts.refferrer_c_record {
                                    let (verify_record_c, _) = get_refferrer_record_key(&refferrer_c.key);
                                    check_account_key(refferrer_c_record, &verify_record_c)?;
                                }

                                let refferrer_c_lamports = share(refferrer_lamports, 13)?;
                                **vault.try_borrow_mut_lamports()? -= refferrer_c_lamports;
                                **refferrer_c.try_borrow_mut_lamports()? += refferrer_c_lamports;
                                
                                msg!("transfer to refferrer C: {:?}", refferrer_c_lamports);

                            } else {
                                msg!("usrB's refferrer is't vault, so B should provide a record account");
                                return Err(ProgramError::InvalidArgument);
                            }
                        }else {
                            msg!("refferrer C is vault");
                            who_vault = 3;
                        }
                    } 

                    msg!("vault: {:?}", vault.key);
                    msg!("vault owner: {:?}", vault.owner);

                    // invoke_signed(
                    //     &system_instruction::transfer(
                    //         vault.key,
                    //         refferrer_b.key,
                    //         share(refferrer_lamports, 26)?,
                    //     ),
                    //     &[
                    //         vault.clone(),
                    //         refferrer_b.clone(),
                    //         accounts.system_program.clone(),
                    //     ],
                    //     &[vault_seeds], 
                    // )?;
                    let refferrer_b_lamports = share(refferrer_lamports, 26)?;
                    **vault.try_borrow_mut_lamports()? -= refferrer_b_lamports;
                    **refferrer_b.try_borrow_mut_lamports()? += refferrer_b_lamports;

                    msg!("transfer to refferrer B: {:?}", refferrer_b_lamports);
                }else {
                    msg!("usrA's refferrer is't vault, so A should provide a record account");
                    return Err(ProgramError::InvalidArgument);
                }

            }else {
                msg!("refferrer B is vault");
                who_vault = 2;
            }
        }else {
            msg!("usr's refferrer is't vault, so A should provide a record account");
            return Err(ProgramError::InvalidArgument);
        }
    }else {
        msg!("refferrer A is vault");
        who_vault = 1;
    }

    if who_vault != 1 {
        
        let refferrer_a_lamports = share(refferrer_lamports, 52)?;
        **vault.try_borrow_mut_lamports()? -= refferrer_a_lamports;
        **accounts.refferrer_a.try_borrow_mut_lamports()? += refferrer_a_lamports;

        msg!("transfer to refferrer A: {:?}", refferrer_a_lamports);
    }

    promotion_inspect(
        who_vault, &accounts, refferrer_lamports, domain_price_lamports
    )?;

    Ok(())
}


