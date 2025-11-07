use solana_program::entrypoint::ProgramResult;
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::program_pack::Pack;
use web3_utils::check::check_account_key;


use crate::processor::settle_auction::Accounts;
use crate::state::{RefferrerRecordHeader, get_refferrer_record_key};
use crate::utils::share;

pub fn settle_qualifications_verify(
    accounts: &Accounts<'_, AccountInfo<'_>>,
    highest_bidder: &Pubkey
) -> ProgramResult {

    if accounts.fee_payer.key == highest_bidder {
        msg!("is the bidder");
        return Ok(())
    }
    if accounts.refferrer_a.key == accounts.fee_payer.key {
        msg!("is the refferrer a");
        return Ok(())
    }
    if let Some(refferr_b) = accounts.refferrer_b {
        if refferr_b.key == accounts.fee_payer.key {
            msg!("is the refferrer b");
            return Ok(())
        }
    }
    if let Some(refferr_c) = accounts.refferrer_c {
        if refferr_c.key == accounts.fee_payer.key {
            msg!("is the refferrer c");
            return Ok(())
        }
    }

    msg!("you have not qualification ro confirm the transaction");
    Err(ProgramError::InvalidArgument)
}

fn refferrer_profit_add(
    refferrer_record: Option<&AccountInfo>,
    profit_add_sol: u64,
    up_level: &mut u8,
    now_user_volumn: u64,
) -> ProgramResult {

    if let Some(refferrer_record) = refferrer_record {
        let mut data_ref = refferrer_record.try_borrow_mut_data()?;
        let mut record_data = 
            RefferrerRecordHeader::unpack_from_slice(&data_ref)?;

        record_data.profit = record_data
            .profit
            .checked_add(profit_add_sol)
            .ok_or(ProgramError::InvalidInstructionData)?;
        msg!("add profit");

        record_data.volume = record_data
            .volume
            .checked_add(profit_add_sol)
            .ok_or(ProgramError::InvalidInstructionData)?;
        msg!("add volumn");

        record_data.pack_into_slice(&mut data_ref);

        if now_user_volumn > record_data.volume {
            *up_level += 1;
        }

    } else {
        msg!("should exist");
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

pub fn promotion_inspect(
    who_vault: u8,
    accounts: &Accounts<'_, AccountInfo<'_>>,
    refferrer_lamports: u64,
    domain_price_lamports: u64,
) -> ProgramResult {

    let user_refferrer_record = accounts.refferrer_record;
    let mut user_record_data = RefferrerRecordHeader::unpack_from_slice(
        &user_refferrer_record.data.borrow()
    )?;

    user_record_data.volume = 
        user_record_data.volume
        .checked_add(domain_price_lamports)
        .ok_or(ProgramError::InvalidInstructionData)?;
    
    let mut up_level: u8 = 0;
    let now_user_volume = user_record_data.volume;

    match who_vault {
        0 => {
            msg!("all not vault");
            refferrer_profit_add(
                accounts.refferrer_c_record,
                share(refferrer_lamports, 13)?,
                &mut up_level,
                now_user_volume,
            )?;
            refferrer_profit_add(
                accounts.refferrer_b_record,
                share(refferrer_lamports, 26)?,
                &mut up_level,
                now_user_volume,
            )?;
            refferrer_profit_add(
                accounts.refferrer_a_record,
                share(refferrer_lamports, 52)?,
                &mut up_level,
                now_user_volume,
            )?;
        }
        1 => {
            msg!("refferrer A is vault");
        }
        2 => {
            msg!("refferrer B is vault");
            refferrer_profit_add(
                accounts.refferrer_a_record,
                share(refferrer_lamports, 52)?,
                &mut up_level,
                now_user_volume,
            )?;
        }
        3 => {
            msg!("refferrer C is vault");
            refferrer_profit_add(
                accounts.refferrer_b_record,
                share(refferrer_lamports, 26)?,
                &mut up_level,
                now_user_volume,
            )?;
            refferrer_profit_add(
                accounts.refferrer_a_record,
                share(refferrer_lamports, 52)?,
                &mut up_level,
                now_user_volume,
            )?;
        }
        _ => {
            return Err(ProgramError::InvalidArgument);
        }
    }

    match up_level {
        0 => {
            msg!("won't promote");
        }
        1 => {
            msg!("uplevel 1");
            if let Some(refferrer_b) = accounts.refferrer_b {
                user_record_data.refferrer_account
                    = *refferrer_b.key
            }else {
                msg!("should has refferrer b");
                return Err(ProgramError::InvalidArgument);
            }
        }
        2 => {
            msg!("uplevel 2");
            if let Some(refferrer_c) = accounts.refferrer_c{
                user_record_data.refferrer_account
                    = *refferrer_c.key
            } else {
                msg!("should has refferrer c");
                return Err(ProgramError::InvalidArgument);
            }
        }
        3 => {
            msg!("uplevel 3");
            if let Some(refferrer_c_record) = accounts.refferrer_c_record{
                let refferrer_c_record_data = RefferrerRecordHeader::unpack_from_slice(
                    &refferrer_c_record.data.borrow()
                )?;

                user_record_data.refferrer_account
                    = refferrer_c_record_data.refferrer_account;
            } else {
                msg!("should has refferrer d");
                return Err(ProgramError::InvalidArgument);
            }
        }
        _ => {
            msg!("error");
            return Err(ProgramError::InvalidArgument);
        }
    }

    user_record_data.pack_into_slice(&mut user_refferrer_record.try_borrow_mut_data()?);


    Ok(())
}

pub fn add_domain_origin_owner_volume(
    accounts: &Accounts<'_, AccountInfo<'_>>,
    domain_price: u64,
) -> ProgramResult {

    let origin_owner = accounts.origin_name_account_owner;
    let origin_owner_refferrer_record = accounts.origin_name_owner_record;

    let (origin_owner_refferrer_record_key, _) = get_refferrer_record_key(origin_owner.key);
    check_account_key(origin_owner_refferrer_record, &origin_owner_refferrer_record_key)?;
   
    let mut origin_owner_record_data = RefferrerRecordHeader::unpack_from_slice(
        &origin_owner_refferrer_record.data.borrow()
    )?;

    let get_lamports = share(domain_price, 95)?;

    msg!("origin owner add: {:?}", origin_owner_record_data.volume);
    
    origin_owner_record_data.profit =
        origin_owner_record_data.profit
        .checked_add(get_lamports)
        .ok_or(ProgramError::InvalidArgument)?;

    origin_owner_record_data.volume =
        origin_owner_record_data.volume
        .checked_add(get_lamports)
        .ok_or(ProgramError::InvalidArgument)?;

    origin_owner_record_data.pack_into_slice(&mut origin_owner_refferrer_record.try_borrow_mut_data()?);

    Ok(())
}