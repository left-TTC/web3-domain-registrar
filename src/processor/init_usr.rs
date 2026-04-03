
use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, msg, program::{invoke, invoke_signed}, program_error::ProgramError, program_pack::Pack, rent::Rent, sysvar::Sysvar
};
use borsh::{BorshDeserialize, BorshSerialize};
use web3_utils::{
    check::{check_account_key, check_signer},
    BorshSize,
    borsh_size::BorshSize,
};
use solana_system_interface::instruction as system_instruction;

use crate::constants::return_vault_key;
use crate::state::vault::VaultRecord;
use crate::state::{ReferrerRecordHeader, get_referrer_record_key};


#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    pub referrer_key: Pubkey,
}

pub fn init_usr (
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let fee_payer = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    let referrer_record = next_account_info(accounts_iter)?;
    let vault = next_account_info(accounts_iter)?;
    let super_referrer_record = next_account_info(accounts_iter).ok();
    
    // Check that system_account is the system program
    check_account_key(system_account, &solana_program::system_program::ID)?;
    
    // Check that fee_payer is a signer
    check_signer(fee_payer)?;

    let (referrer_record_key, seed) = get_referrer_record_key(fee_payer.key);
    check_account_key(referrer_record, &referrer_record_key)?;

    let (vault_key, _) = return_vault_key();
    check_account_key(vault, &vault_key)?;

    if !referrer_record.data_is_empty() {
        msg!("has registered");
        return Err(ProgramError::InvalidArgument);
    }

    let (vault_key, _) = return_vault_key();
    if params.referrer_key != vault_key {
        msg!("use other's referrer key");
        let (super_referrer_key, _) = get_referrer_record_key(&params.referrer_key);
        match super_referrer_record {
            Some(account) => {
                check_account_key(account, &super_referrer_key)?;
                ReferrerRecordHeader::unpack_from_slice(&account.data.borrow())?;
            }
            None => {
                msg!("should got an super referrer");
                return Err(ProgramError::InvalidArgument);
            }
        }
    }

    // Create referrer record account if it doesn't exist
    let rent = Rent::get()?;
    let referrer_record_lamports = rent.minimum_balance(ReferrerRecordHeader::LEN);
    
    // Transfer lamports to create the account
    invoke(
        &system_instruction::transfer(
            fee_payer.key, 
            &referrer_record_key, 
            referrer_record_lamports
        ), 
        &[
            fee_payer.clone(),
            referrer_record.clone(),
            system_account.clone(),
        ],
    )?;

    // Allocate space for the account
    invoke_signed(
        &system_instruction::allocate(
            &referrer_record_key, 
            ReferrerRecordHeader::LEN as u64
        ), 
        &[referrer_record.clone(), system_account.clone()], 
        &[&seed.chunks(32).collect::<Vec<&[u8]>>()],
    )?;

    // Assign the account to our program
    invoke_signed(
        &system_instruction::assign(&referrer_record_key, &crate::ID),
        &[referrer_record.clone(), system_account.clone()],
        &[&seed.chunks(32).collect::<Vec<&[u8]>>()],
    )?;

    // Initialize the referrer record data
    let clock = Clock::get()?;
    let referrer_record_data = ReferrerRecordHeader::new(
        params.referrer_key,
        clock.unix_timestamp,
    );
    
    referrer_record_data.pack_into_slice(&mut referrer_record.data.borrow_mut());
    msg!("Referrer record created successfully");

    let mut vault_record = 
        VaultRecord::unpack_from_slice(&vault.data.borrow())?;
    vault_record.usr_count = vault_record.usr_count.checked_add(1)
        .ok_or(ProgramError::InvalidArgument)?;
    msg!("add a usr count");

    Ok(())
}