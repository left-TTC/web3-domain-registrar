use web3_utils::{
    BorshSize, InstructionsAccount, accounts::InstructionsAccount, borsh_size::BorshSize, check::{check_account_key, check_account_owner, check_signer}
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_pack::Pack,
    account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey
};

use crate::{constants::return_vault_key, state::{ReferrerRecordHeader, get_referrer_record_key}, utils::{math, share_with_cap}};

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub extraction: u64
}


#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> { 
    #[cons(writable, signer)]
    pub user: &'a T,
    #[cons(writable)]
    pub user_referrer_record: &'a T,
    #[cons(writable)]
    pub vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            user:next_account_info(accounts_iter)?,
            user_referrer_record: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_owner(self.user_referrer_record, &crate::ID)?;

        check_signer(self.user).unwrap();
        msg!("user signature ok");

        Ok(())
    }
}

pub fn process_extract<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;
    accounts.check()?;

    msg!("use withdraw {} lamports", params.extraction);

    let (referrer_key, _) = get_referrer_record_key(accounts.user.key);
    check_account_key(accounts.user_referrer_record, &referrer_key)?;
    msg!("referrer key ok");

    let (vault_key, _) = return_vault_key();
    check_account_key(accounts.vault, &vault_key)?;
    msg!("vault key ok");

    let mut data_ref = accounts.user_referrer_record.try_borrow_mut_data()?;
    let mut record_data = 
        ReferrerRecordHeader::unpack_from_slice(&data_ref)?;

    // devnet - 0.01SOL Mainnet - 0.1SOL 
    if math::sub(record_data.profit, params.extraction)? <= 10_000_000 {
        msg!("should leave 0.01SOL or 0.1SOL");
        return Err(ProgramError::InvalidArgument);
    }

    let real_ex = share_with_cap(params.extraction, 990_000_000)?;

    **accounts.vault.try_borrow_mut_lamports()? -= real_ex;
    **accounts.user.try_borrow_mut_lamports()? += real_ex;
    msg!("transfer ok");

    record_data.profit = math::sub(record_data.profit, params.extraction)?;
    record_data.pack_into_slice(&mut data_ref); 

    Ok(())
}