//! Create a domain name and buy the ownership of a domain name

use web3_utils::{
    accounts::InstructionsAccount, 
    borsh_size::BorshSize, 
    check::{check_account_key, check_account_owner, check_signer}, 
    BorshSize, 
    InstructionsAccount
};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    clock::Clock, entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError, 
    program_pack::Pack, 
    pubkey::Pubkey, 
    sysvar::{self, Sysvar}
};
use web3_domain_name_service::{utils::get_seeds_and_key};

use crate::{central_state, constants::{SYSTEM_ID, WEB3_NAME_SERVICE}, state::{write_data, NameStateRecordHeader}, utils::get_hashed_name};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    my_price: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> {
    /// The naming service program ID
    pub naming_service_program: &'a T,
    /// The root domain account       
    pub root_domain: &'a T,
    /// The name account
    #[cons(writable)]
    pub name: &'a T,
    /// The reverse look up account   
    #[cons(writable)]
    pub reverse_lookup: &'a T,
    /// The domain auction state account
    #[cons(writable)]
    pub domain_state_account: &'a T,
    /// The system program account
    pub system_program: &'a T,
    /// The central state account
    pub central_state: &'a T,
    /// The buyer account         
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    /// The Pyth feed account
    pub pyth_feed_account: &'a T,
    /// The rent sysvar account
    pub rent_sysvar: &'a T,
    // it's not necessary to confirm the referrer
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            naming_service_program: next_account_info(accounts_iter)?,
            root_domain: next_account_info(accounts_iter)?,
            name: next_account_info(accounts_iter)?,
            reverse_lookup: next_account_info(accounts_iter)?,
            domain_state_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            pyth_feed_account: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_key(self.naming_service_program, &WEB3_NAME_SERVICE).unwrap();
        msg!("nameservice id ok");
        check_account_key(self.system_program, &SYSTEM_ID).unwrap();
        msg!("system_program id ok");
        check_account_key(self.central_state, &central_state::KEY).unwrap();
        msg!("central_state id ok");
        check_account_key(self.rent_sysvar, &sysvar::rent::ID).unwrap();
        msg!("rent_sysvar id ok");

        // Check ownership
        check_account_owner(self.name, &SYSTEM_ID)
            .map_err(|_| crate::Error::AlreadyRegistered)?;
        check_account_owner(self.root_domain, &WEB3_NAME_SERVICE)?;
        check_account_owner(self.domain_state_account, &crate::ID)
            .map_err(|_| crate::Error::AlreadyRegistered)?;

        check_signer(self.fee_payer).unwrap();
        msg!("fee_payer signature ok");

        Ok(())
    }
}


pub fn process_increase_price<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;
    accounts.check()?;

    let (name_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key, 
        get_hashed_name(&params.name), 
        None, 
        Some(accounts.root_domain.key)
    );

    msg!("name account: {}", name_account_key);

    if &name_account_key != accounts.name.key {
        msg!("Provided wrong name account");
        return Err(ProgramError::InvalidArgument);
    }

    let hashed_reverse_lookup = get_hashed_name(&name_account_key.to_string());

    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_reverse_lookup.clone(),
        Some(&central_state::KEY),
        None,
    );

    if &reverse_lookup_account_key != accounts.reverse_lookup.key {
        msg!("Provided wrong reverse lookup account");
        return Err(ProgramError::InvalidArgument);
    }

    //auction state
    let (name_state_key, name_state_seeds) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&params.name), 
        Some(&central_state::KEY), 
        Some(accounts.root_domain.key)
    );
    
    let name_state_account = accounts.domain_state_account;
    if name_state_key != *name_state_account.key {
        msg!("The given name state account is incorrect.");
        return Err(ProgramError::InvalidArgument);
    }

    let name_state_data = 
        NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;

    write_data(accounts.domain_state_account, &accounts.fee_payer.key.to_bytes(), 0);

    let update_time = Clock::get()?.unix_timestamp;
    write_data(accounts.domain_state_account, &update_time.to_le_bytes(), 64);
    
    let new_price: u64 = name_state_data.highest_price + params.my_price;
    write_data(accounts.domain_state_account, &new_price.to_le_bytes(), 72);

    Ok(())
}
