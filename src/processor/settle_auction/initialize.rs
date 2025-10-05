
use solana_program::{
    account_info::{AccountInfo}, entrypoint::ProgramResult, msg, program::invoke, program_error::ProgramError, program_pack::Pack, rent::Rent, sysvar::Sysvar 
};

use solana_system_interface::instruction as system_instruction;
use web3_domain_name_service::{state::NameRecordHeader, utils::get_seeds_and_key};
use web3_utils::check::check_account_key;

use crate::{central_state, cpi::Cpi, state::RefferrerRecordHeader, utils::{get_hashed_name, share}};

pub fn initialize_settle(
    accounts: super::Accounts<'_, AccountInfo<'_>>,
    params: super::Params,
    total_price: u64,
    hased_name: Vec<u8>,
    hashed_reverse_lookup: Vec<u8>,
) -> ProgramResult {

    let (vault_key, _) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name("vault"), 
        Some(&central_state::KEY), 
        Some(&central_state::KEY)
    );

    // check fee payer's refferrer record
    let (refferrer_record_key, _) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&accounts.fee_payer.key.to_string()), 
        Some(&crate::ID), 
        Some(&crate::ID),
    );
    check_account_key(accounts.refferrer_record, &refferrer_record_key)?;

    let refferrer_usr_data = 
        RefferrerRecordHeader::unpack_from_slice(&accounts.refferrer_record.data.borrow())?;

    if &refferrer_usr_data.refferrer_account != accounts.refferrer_a.key{
        msg!("the refferrer one is error");
        return Err(ProgramError::InvalidArgument);
    }

    let mut one_ratio: u64 = 90;

    if refferrer_usr_data.refferrer_account != vault_key{
        // if refferrer is vault => means no super level -- settle directly

        // != vault_key => means A has refferrer too
        if let Some(refferrer_a_record) = accounts.refferrer_a_record{
            let (verify_record_a, _) = get_seeds_and_key(
                &crate::ID, 
                get_hashed_name(&accounts.refferrer_a.key.to_string()), 
                Some(&crate::ID), 
                Some(&crate::ID),
            );
            check_account_key(refferrer_a_record, &verify_record_a)?;

            // get recorded refferrer: B
            let a_record_data = 
                RefferrerRecordHeader::unpack_from_slice(&refferrer_a_record.data.borrow())?;
            
            if a_record_data.refferrer_account != vault_key {
                if let Some(refferrer_b) = accounts.refferrer_b{
                    
                    if let Some(refferrer_b_record) = accounts.refferrer_b_record{
                        // get B's refferrer record account
                        let (verify_record_b, _) = get_seeds_and_key(
                            &crate::ID, 
                            get_hashed_name(&refferrer_b.key.to_string()), 
                            Some(&crate::ID), 
                            Some(&crate::ID),
                        );
                        check_account_key(refferrer_b_record, &verify_record_b)?;
                        msg!("refferrer B's record is exsited");

                        // get B's refferrer: C
                        let b_record_data = 
                            RefferrerRecordHeader::unpack_from_slice(&refferrer_b_record.data.borrow())?;

                        if b_record_data.refferrer_account != vault_key {
                            if let Some(refferrer_c) = accounts.refferrer_c {
                                invoke(&system_instruction::transfer(
                                    accounts.fee_payer.key, refferrer_c.key, share(total_price, 20)?), 
                                    &[
                                        accounts.fee_payer.clone(),
                                        refferrer_c.clone(),
                                        accounts.system_program.clone(),
                                    ]
                                )?;
                                msg!("transfer to refferrer C ok");

                                one_ratio -= 20;
                            }else {
                                msg!("B's refferrer is't vault, so there must be an refferrer C");
                                return Err(ProgramError::InvalidArgument);
                            }
                        }
                    }else {
                        msg!("B is exsited, so there must be an refferrer B record");
                        return Err(ProgramError::InvalidArgument);
                    }

                    invoke(&system_instruction::transfer(
                        accounts.fee_payer.key, refferrer_b.key, share(total_price, 30)?), 
                        &[
                            accounts.fee_payer.clone(),
                            refferrer_b.clone(),
                            accounts.system_program.clone(),
                        ]
                    )?;
                    msg!("transfer to refferrer B ok");
                    one_ratio -= 30;
                }else {
                    msg!("A's refferrer is't vault, so there must be an refferrer B");
                    return Err(ProgramError::InvalidArgument);
                }
            }

        }else {
            msg!("usr's refferrer is't vault, so A should provide a record account");
            return Err(ProgramError::InvalidArgument);
        }
    }

    invoke(&system_instruction::transfer(
        accounts.fee_payer.key, accounts.refferrer_a.key, share(total_price, one_ratio)?), 
        &[
            accounts.fee_payer.clone(),
            accounts.refferrer_a.clone(),
            accounts.system_program.clone(),
        ]
    )?;
    msg!("transfer to refferrer A ok");

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
            accounts.rent_sysvar, 
            central_state_signer_seeds, 
            None, 
            None
        )?;
    }
    
    Ok(())
}