use web3_utils::{
    check::{check_account_key, check_account_owner, check_signer},
    BorshSize, InstructionsAccount,
    borsh_size::BorshSize,
    accounts::InstructionsAccount,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent, sysvar::{self, Sysvar}
};
use web3_domain_name_service::{state::NameRecordHeader, utils::get_seeds_and_key};
use solana_system_interface::instruction as system_instruction;

use crate::{central_state, constants::{ADMIN_ANDY, ADMIN_FANMOCHENG, SYSTEM_ID, return_vault_key}, 
    state::{NameStateRecordHeader, RefferrerRecordHeader, ReverseLookup, get_refferrer_record_key}, 
    utils::{TIME, check_state_time, get_hashed_name, get_now_time, share}
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub extraction_volume: u64
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> { 
    #[cons(writable, signer)]
    pub admin_signer: &'a T,
    pub admin_other: &'a T,
    #[cons(writable)]
    pub vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            admin_signer:next_account_info(accounts_iter)?,
            admin_other: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        let admin_one = self.admin_signer.key;
        if admin_one != &ADMIN_ANDY && admin_one != &ADMIN_FANMOCHENG {
            msg!("admin error");
            return Err(ProgramError::InvalidArgument);
        }
        let admin_two = self.admin_other.key;
        if admin_two != &ADMIN_ANDY && admin_two != &ADMIN_FANMOCHENG {
            msg!("admin error");
            return Err(ProgramError::InvalidArgument);
        }

        // Check signer
        check_signer(self.admin_signer).unwrap();
        msg!("fee_payer signature ok");

        Ok(())
    }
}

pub fn process_start_name<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;
    accounts.check()?;

    let transfer_out_lamports = share(params.extraction_volume, 50)?;

    **accounts.vault.try_borrow_mut_lamports()? -= transfer_out_lamports * 2;

    **accounts.admin_signer.try_borrow_mut_lamports()? += transfer_out_lamports;
    **accounts.admin_other.try_borrow_mut_lamports()? += transfer_out_lamports;


    Ok(())
}