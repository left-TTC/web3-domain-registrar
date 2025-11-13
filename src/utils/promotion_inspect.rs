use solana_program::entrypoint::ProgramResult;
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::program_pack::Pack;


use crate::processor::settle_auction::Accounts;
use crate::state::{ReferrerRecordHeader};
use crate::utils::share;

pub fn settle_qualifications_verify(
    accounts: &Accounts<'_, AccountInfo<'_>>,
    highest_bidder: &Pubkey
) -> ProgramResult {

    if accounts.fee_payer.key == highest_bidder {
        msg!("is the bidder");
        return Ok(())
    }
    if accounts.referrer_a.key == accounts.fee_payer.key {
        msg!("is the referrer a");
        return Ok(())
    }
    if let Some(refferr_b) = accounts.referrer_b {
        if refferr_b.key == accounts.fee_payer.key {
            msg!("is the referrer b");
            return Ok(())
        }
    }
    if let Some(refferr_c) = accounts.referrer_c {
        if refferr_c.key == accounts.fee_payer.key {
            msg!("is the referrer c");
            return Ok(())
        }
    }

    msg!("you have not qualification ro confirm the transaction");
    Err(ProgramError::InvalidArgument)
}

fn referrer_profit_add(
    referrer_record: Option<&AccountInfo>,
    // These are the subordinates's shares
    profit_add_sol: u64,
) -> Result<u64, ProgramError> {

    if let Some(referrer_record) = referrer_record {
        let mut data_ref = referrer_record.try_borrow_mut_data()?;
        let mut record_data = 
            ReferrerRecordHeader::unpack_from_slice(&data_ref)?;

        record_data.profit = record_data
            .profit
            .checked_add(profit_add_sol)
            .ok_or(ProgramError::InsufficientFunds)?;
        msg!("add profit");

        record_data.performance = record_data
            .performance
            .checked_add(profit_add_sol)
            .ok_or(ProgramError::InsufficientFunds)?;
        msg!("add volumn");

        record_data.pack_into_slice(&mut data_ref);

        return Ok(record_data.performance);
    }

    msg!("should exist");
    return Err(ProgramError::InvalidArgument);
    
}

fn up_level_to(
    record_will_up: Option<&AccountInfo>,
    record_the_levfel: Option<&AccountInfo>
) -> ProgramResult {

    if let (Some(record_up), Some(record_level)) = (record_will_up, record_the_levfel){
        let new_referrer = 
            ReferrerRecordHeader::unpack_from_slice(&record_level.try_borrow_data()?)?.referrer_account;

        let mut data_ref = record_up.try_borrow_mut_data()?;
        let mut record_up_data = 
            ReferrerRecordHeader::unpack_from_slice(&data_ref)?;
        
        record_up_data.referrer_account = new_referrer;
        record_up_data.pack_into_slice(&mut data_ref);

        msg!("update to {:?}", new_referrer);

        Ok(())
    }else {
        msg!("shoul provide the two accounts");
        return Err(ProgramError::InvalidArgument);  
    } 
}

/// Check if referrer A and referrer B need to upgrade, and then perform the upgrade operation.
pub fn promotion_inspect(
    // which referrer is vault
    // 0 -- no vault 1 -- A 2 -- B 3 -- C 
    who_vault: u8,
    accounts: &Accounts<'_, AccountInfo<'_>>,
    referrer_lamports: u64,
) -> ProgramResult {

    match who_vault {
        0 => {
            // chain new_owner -> A -> B -> C -> ...
            msg!("all not vault");
            let a_performance = referrer_profit_add(
                accounts.referrer_a_record,
                share(referrer_lamports, 52)?,
            )?;

            let b_performance = referrer_profit_add(
                accounts.referrer_b_record,
                share(referrer_lamports, 26)?,
            )?;
            
            let c_performance = referrer_profit_add(
                accounts.referrer_c_record,
                share(referrer_lamports, 13)?
            )?;

            // check b frist 
            if b_performance > c_performance {
                up_level_to(
                    accounts.referrer_b_record, 
                    accounts.referrer_c_record
                )?;
                msg!("up b to c's level")
            }

            if a_performance > b_performance {
                up_level_to(
                    accounts.referrer_a_record, 
                    accounts.referrer_b_record
                )?;
            }
        }
        1 => {
            msg!("referrer A is vault");
        }
        2 => {
            msg!("referrer B is vault, means A is highest level");
            referrer_profit_add(
                accounts.referrer_a_record,
                share(referrer_lamports, 52)?
            )?;
        }
        3 => {
            msg!("referrer C is vault, only check wheather A is going to up level");
            let a_performance = referrer_profit_add(
                accounts.referrer_a_record,
                share(referrer_lamports, 52)?,
            )?;

            let b_performance = referrer_profit_add(
                accounts.referrer_b_record,
                share(referrer_lamports, 26)?,
            )?;

            if a_performance > b_performance {
                up_level_to(
                    accounts.referrer_a_record, 
                    accounts.referrer_b_record
                )?;
            }
        }
        _ => {
            return Err(ProgramError::InvalidArgument);
        }
    }

    Ok(())
}
