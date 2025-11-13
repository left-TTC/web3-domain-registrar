
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError, program_pack::Pack
};

use web3_utils::check::check_account_key;

use crate::{state::{ReferrerRecordHeader, get_referrer_record_key}, utils::{promotion_inspect::promotion_inspect}};


// 11.10 changed: cancle all directly transfer SOL

pub fn transfer_by_referrer_chain(
    accounts: &crate::processor::settle_auction::Accounts<'_, AccountInfo<'_>>,
    referrer_lamports: u64,
) -> ProgramResult {

    let vault = accounts.vault;
    

    let domain_owner = accounts.new_domain_owner;
    let (owner_record, _) = get_referrer_record_key(domain_owner.key);
    check_account_key(accounts.referrer_record, &owner_record)?;

    let referrer_usr_data = 
        ReferrerRecordHeader::unpack_from_slice(&accounts.referrer_record.data.borrow())?;

    let mut who_vault: u8 = 0;

    if &referrer_usr_data.referrer_account != vault.key{

        if &referrer_usr_data.referrer_account != accounts.referrer_a.key {
            msg!("provide fault referrer A");
            return Err(ProgramError::InvalidArgument);
        }

        if let Some(referrer_a_record) = accounts.referrer_a_record {

            let (verify_record_a, _) = get_referrer_record_key(&accounts.referrer_a.key);
            check_account_key(referrer_a_record, &verify_record_a)?;

            let a_record_data = 
                ReferrerRecordHeader::unpack_from_slice(&referrer_a_record.data.borrow())?;

            if &a_record_data.referrer_account != vault.key {
                
                if let Some(referrer_b) = accounts.referrer_b {
                    if &a_record_data.referrer_account != referrer_b.key {
                        msg!("provide fault referrer B");
                        return Err(ProgramError::InvalidArgument);
                    }

                    if let Some(referrer_b_record) = accounts.referrer_b_record {

                        let (verify_record_b, _) = get_referrer_record_key(&referrer_b.key);
                        check_account_key(referrer_b_record, &verify_record_b)?;

                        let b_record_data = 
                            ReferrerRecordHeader::unpack_from_slice(&referrer_b_record.data.borrow())?;

                        if &b_record_data.referrer_account != vault.key {

                            if let Some(referrer_c) = accounts.referrer_c {

                                if &b_record_data.referrer_account != referrer_c.key {
                                    msg!("provide fault referrer C");
                                    return Err(ProgramError::InvalidArgument);
                                }

                                if let Some(referrer_c_record)  = accounts.referrer_c_record {
                                    let (verify_record_c, _) = get_referrer_record_key(&referrer_c.key);
                                    check_account_key(referrer_c_record, &verify_record_c)?;
                                }

                            } else {
                                msg!("usrB's referrer is't vault, so B should provide a record account");
                                return Err(ProgramError::InvalidArgument);
                            }
                        }else {
                            msg!("referrer C is vault");
                            who_vault = 3;
                        }
                    } 

                    msg!("vault: {:?}", vault.key);
                    msg!("vault owner: {:?}", vault.owner);
                }else {
                    msg!("usrA's referrer is't vault, so A should provide a record account");
                    return Err(ProgramError::InvalidArgument);
                }

            }else {
                msg!("referrer B is vault");
                who_vault = 2;
            }
        }else {
            msg!("usr's referrer is't vault, so A should provide a record account");
            return Err(ProgramError::InvalidArgument);
        }
    }else {
        msg!("referrer A is vault");
        who_vault = 1;
    }

    promotion_inspect(
        who_vault, 
        &accounts, 
        referrer_lamports,
    )?;

    Ok(())
}


