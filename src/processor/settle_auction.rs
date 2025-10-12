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
    account_info::{next_account_info, AccountInfo}, sysvar, entrypoint::ProgramResult, msg, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey
};
use web3_domain_name_service::{state::NameRecordHeader, utils::get_seeds_and_key};


use crate::{central_state, constants::{SYSTEM_ID}, state::NameStateRecordHeader, utils::{check_state_time, get_hashed_name, get_sol_price, TIME}};

pub mod initialize;
pub mod repeat;

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub domain_name: String,
    pub custom_price: Option<u64>,
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
    pub domain_state_account: &'a T,
    /// The system program account
    pub system_program: &'a T,
    /// cnetral state register
    pub central_state: &'a T,
    /// The buyer account         
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    /// The Pyth feed account
    pub pyth_feed_account: &'a T,
    /// rent sysvar
    pub rent_sysvar: &'a T,
    /// name account owner -- The initialized domain name can be arbitrary
    /// Domain names auctioned more than twice must be the same as in the records
    pub name_account_owner: &'a T,
    /// buyer's refferrer record
    pub refferrer_record: &'a T,
    /// buyer's refferrer -- we named A
    #[cons(writable)]
    pub refferrer_a: &'a T,
    /// A's refferrer record
    pub refferrer_a_record: Option<&'a T>,
    /// A's refferrer -- named B
    #[cons(writable)]
    pub refferrer_b: Option<&'a T>,
    /// B's refferrer record
    pub refferrer_b_record: Option<&'a T>,
    /// B's refferrer -- named C
    #[cons(writable)]
    pub refferrer_c: Option<&'a T>,
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
            name_account_owner: next_account_info(accounts_iter)?,
            refferrer_record: next_account_info(accounts_iter)?,
            refferrer_a: next_account_info(accounts_iter)?,
            refferrer_a_record: next_account_info(accounts_iter).ok(),
            refferrer_b: next_account_info(accounts_iter).ok(),
            refferrer_b_record: next_account_info(accounts_iter).ok(),
            refferrer_c: next_account_info(accounts_iter).ok(),
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_key(self.naming_service_program, &web3_domain_name_service::ID).unwrap();
        msg!("nameservice id ok");
        check_account_key(self.system_program, &SYSTEM_ID).unwrap();
        msg!("system_program id ok");
        check_account_key(self.rent_sysvar, &sysvar::rent::ID)?;

        check_account_owner(self.root_domain, &web3_domain_name_service::ID)?;
        check_account_owner(self.domain_state_account, &crate::ID)?;

        check_signer(self.fee_payer).unwrap();
        msg!("fee_payer signature ok");

        Ok(())
    }
}


pub fn process_settle_auction<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;
    accounts.check()?;

    let name_state_account = accounts.domain_state_account;
    let hased_name = get_hashed_name(&params.domain_name);
    let (name_state_key, _) = get_seeds_and_key(
        &crate::ID, 
        hased_name.clone(), 
        Some(&central_state::KEY), 
        Some(accounts.root_domain.key)
    );
    check_account_key(name_state_account, &name_state_key)?;
    msg!("name state key ok");
    let name_state = 
        NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
    
    if check_state_time(name_state.update_time)? != TIME::SETTLE {
        msg!("over settle time");
        return Err(ProgramError::InvalidArgument);
    }
    
    check_account_key(accounts.fee_payer, &name_state.highest_bidder)?;
    msg!("settle man right");

    let domain_name_account = accounts.name;
    let (name_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key, 
        hased_name.clone(), 
        None, 
        Some(accounts.root_domain.key)
    );
    check_account_key(domain_name_account, &name_account_key)?;
    msg!("name account key ok");

    let hashed_reverse_lookup = get_hashed_name(&name_account_key.to_string());
    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_reverse_lookup.clone(),
        Some(&central_state::KEY),
        None,
    );
    check_account_key(accounts.reverse_lookup, &reverse_lookup_account_key)?;
    msg!("reverse account key ok");

    // have already paid 10% or 5% -- check name account to distinguish between these two cases
    let price = get_sol_price(accounts.pyth_feed_account, name_state.highest_price)?;

    let name_record = 
        NameRecordHeader::unpack_from_slice(&domain_name_account.data.borrow());

    match name_record {
        // buy from others -- means deposit ratio is 5%
        Ok(record_data) =>{
            self::repeat::repeat_settle(
                accounts, 
                params, 
                record_data, 
                name_state, 
                price, 
            )?;
        }
        Err(_) => {
            self::initialize::initialize_settle(
                accounts, 
                params, 
                price, 
                hased_name, 
                hashed_reverse_lookup
            )?;
        }
    }

    Ok(())
}
