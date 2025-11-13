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


use crate::{central_state, constants::SYSTEM_ID, state::{NameStateRecordHeader, get_name_state_key}, utils::{TIME, check_state_time, get_hashed_name, promotion_inspect::settle_qualifications_verify}};

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
    #[cons(writable)]
    pub domain_state_account: &'a T,
    /// The system program account
    pub system_program: &'a T,
    /// cnetral state register
    pub central_state: &'a T,
    /// The buyer account         
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    /// rent sysvar
    pub rent_sysvar: &'a T,
    /// name account owner -- The initialized domain name can be arbitrary
    /// Domain names auctioned more than twice must be the same as in the records
    #[cons(writable)]
    pub origin_name_account_owner: &'a T,
    #[cons(writable)]
    pub origin_name_owner_record: &'a T,
    /// vault
    #[cons(writable)]
    pub vault: &'a T,
    /// new domain owner
    pub new_domain_owner: &'a T,
    /// new owner's referrer record
    #[cons(writable)]
    pub referrer_record: &'a T,
    /// buyer's referrer -- we named A
    #[cons(writable)]
    pub referrer_a: &'a T,
    /// A's referrer record
    #[cons(writable)]
    pub referrer_a_record: Option<&'a T>,
    /// A's referrer -- named B
    #[cons(writable)]
    pub referrer_b: Option<&'a T>,
    /// B's referrer record
    #[cons(writable)]
    pub referrer_b_record: Option<&'a T>,
    /// B's referrer -- named C
    #[cons(writable)]
    pub referrer_c: Option<&'a T>,
    /// C's referrer record
    #[cons(writable)]
    pub referrer_c_record: Option<&'a T>,
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
            rent_sysvar: next_account_info(accounts_iter)?,
            origin_name_account_owner: next_account_info(accounts_iter)?,
            origin_name_owner_record: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            new_domain_owner:next_account_info(accounts_iter)?,
            referrer_record: next_account_info(accounts_iter)?,
            referrer_a: next_account_info(accounts_iter)?,
            referrer_a_record: next_account_info(accounts_iter).ok(),
            referrer_b: next_account_info(accounts_iter).ok(),
            referrer_b_record: next_account_info(accounts_iter).ok(),
            referrer_c: next_account_info(accounts_iter).ok(),
            referrer_c_record: next_account_info(accounts_iter).ok(),
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

// all pepole on the referrer chain can confirm the domain

pub fn process_settle_auction<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;
    accounts.check()?;

    let name_state_account = accounts.domain_state_account;
    let hased_name = get_hashed_name(&params.domain_name);

    let (name_state_key, _) = get_name_state_key(
        &params.domain_name, 
        accounts.root_domain.key
    );
    check_account_key(name_state_account, &name_state_key)?;
    msg!("name state key ok");

    {    
        let name_state_data = 
            NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
        
        // after auction time 
        if check_state_time(name_state_data.update_time)? != TIME::PENDING {
            msg!("not settle time");
            return Err(ProgramError::InvalidArgument);
        }
        
        settle_qualifications_verify(&accounts, &name_state_data.highest_bidder)?;
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
    
        let name_record = 
            NameRecordHeader::unpack_from_slice(&domain_name_account.data.borrow());

        match name_record {
            // buy from others -- means deposit ratio is 5%
            Ok(record_data) =>{
                self::repeat::repeat_settle(
                    accounts, 
                    params, 
                    record_data, 
                    &name_state_data, 
                )?;
            }
            Err(_) => {
                self::initialize::initialize_settle(
                    accounts, 
                    params, 
                    &name_state_data, 
                    hased_name, 
                    hashed_reverse_lookup
                )?;
            }
        }
    }
    

    let mut name_state_data =
        NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
    name_state_data.highest_price += 1;
    name_state_data.settled = true;
    name_state_data.pack_into_slice(&mut name_state_account.try_borrow_mut_data()?);

    Ok(())
}
