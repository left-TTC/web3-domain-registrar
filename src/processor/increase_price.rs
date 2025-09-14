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
    entrypoint::ProgramResult, 
    msg, 
    program::invoke, 
    program_error::ProgramError, 
    program_pack::Pack, 
    pubkey::Pubkey, 
};
use web3_domain_name_service::{utils::get_seeds_and_key};

use solana_system_interface::instruction as system_instruction;

use crate::{central_state, constants::{SYSTEM_ID, WEB3_NAME_SERVICE}, state::{write_data, NameStateRecordHeader}, utils::{check_state_time, get_hashed_name, get_now_time, get_sol_price, AUCTION_DEPOSIT, TIME}};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    pub my_price: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> {
    /// The naming service program ID
    pub naming_service_program: &'a T,
    /// The root domain account       
    pub root_domain: &'a T,
    /// The domain auction state account
    #[cons(writable)]
    pub domain_state_account: &'a T,
    /// The system program account
    pub system_program: &'a T,
    /// The buyer account         
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    /// The Pyth feed account
    pub pyth_feed_account: &'a T,
    // it's not necessary to confirm the referrer
    /// last bidder
    #[cons(writable)]
    pub last_bidder: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            naming_service_program: next_account_info(accounts_iter)?,
            root_domain: next_account_info(accounts_iter)?,
            domain_state_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            pyth_feed_account: next_account_info(accounts_iter)?,
            last_bidder: next_account_info(accounts_iter)?,
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_key(self.naming_service_program, &WEB3_NAME_SERVICE).unwrap();
        msg!("nameservice id ok");
        check_account_key(self.system_program, &SYSTEM_ID).unwrap();
        msg!("system_program id ok");

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

    let name_state_account = accounts.domain_state_account;
    let name_state = 
        NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
    
    if check_state_time(name_state.update_time)? != TIME::AUCTION {
        msg!("This auction has settled");
        return Err(ProgramError::InvalidArgument);
    }

    if params.my_price <= name_state.highest_price{
        msg!("You should bid more than the original bid");
        return Err(ProgramError::InvalidArgument);
    }

    //auction state
    let name_state_account = accounts.domain_state_account;
    let (name_state_key, _) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&params.name), 
        Some(&central_state::KEY), 
        Some(accounts.root_domain.key)
    );
    check_account_key(name_state_account, &name_state_key)?;
    
    // transfer back the deposit
    let back_deposit = get_sol_price(accounts.pyth_feed_account, AUCTION_DEPOSIT)?;
    invoke(&system_instruction::transfer(
        accounts.fee_payer.key, accounts.last_bidder.key, back_deposit), 
        &[
            accounts.fee_payer.clone(),
            accounts.last_bidder.clone(),
            accounts.system_program.clone(),
        ]
    )?;

    write_data(accounts.domain_state_account, &accounts.fee_payer.key.to_bytes(), 0);

    let update_time = get_now_time()?;
    write_data(accounts.domain_state_account, &update_time.to_le_bytes(), 64);
    
    let new_price: u64 = name_state.highest_price + params.my_price;
    write_data(accounts.domain_state_account, &new_price.to_le_bytes(), 72);

    Ok(())
}
