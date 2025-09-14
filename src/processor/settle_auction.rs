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
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar, 
};
use web3_domain_name_service::{state::NameRecordHeader, utils::get_seeds_and_key};

use solana_system_interface::instruction as system_instruction;

use crate::{central_state, constants::{SYSTEM_ID, WEB3_NAME_SERVICE}, cpi::Cpi, state::NameStateRecordHeader, utils::{check_state_time, get_hashed_name, get_now_time, get_sol_price, AUCTION_DEPOSIT, TIME}};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
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
    /// The Pyth feed account
    pub pyth_feed_account: &'a T,
    /// rent sysvar
    pub rent_sysvar: &'a T,
    /// state account rent payer
    #[cons(writable)]
    pub state_rent_payer: &'a T,
    /// buyer's referrer -- we named A
    #[cons(writable)]
    pub referrer_one: &'a T,
    /// vault
    #[cons(writable)]
    pub vault: &'a T,
    /// A's refferrer -- named B
    #[cons(writable)]
    pub referrer_two: Option<&'a T>,
    /// B's referrer -- named C
    #[cons(writable)]
    pub referrer_three: Option<&'a T>,
    
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
            state_rent_payer: next_account_info(accounts_iter)?,
            referrer_one: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            referrer_two: accounts_iter.next(),
            referrer_three: accounts_iter.next(),
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_key(self.naming_service_program, &WEB3_NAME_SERVICE).unwrap();
        msg!("nameservice id ok");
        check_account_key(self.system_program, &SYSTEM_ID).unwrap();
        msg!("system_program id ok");

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


pub fn process_settle_auction<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;
    accounts.check()?;

    let name_state_account = accounts.domain_state_account;
    let name_state = 
        NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
    
    if check_state_time(name_state.update_time)? != TIME::SETTLE {
        msg!("over settle time");
        return Err(ProgramError::InvalidArgument);
    }
    
    check_account_key(accounts.fee_payer, &name_state.highest_bidder)?;
    msg!("settle man right");

    let hashed_name = get_hashed_name(&params.name);

    let (name_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key, 
        get_hashed_name(&params.name), 
        None, 
        Some(accounts.root_domain.key)
    );
    check_account_key(accounts.name, &name_account_key)?;
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

    //auction state
    let name_state_account = accounts.domain_state_account;
    let (name_state_key, _) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&params.name), 
        Some(&central_state::KEY), 
        Some(accounts.root_domain.key)
    );
    check_account_key(name_state_account, &name_state_key)?;
    msg!("name state key ok");

    let vault = accounts.vault;
    let (vault_key, _) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name("vault"), 
        Some(&central_state::KEY), 
        Some(&central_state::KEY)
    );
    check_account_key(vault, &vault_key)?;
    msg!("vault key ok");
    
    // transfer back the deposit
    let fee_payer = accounts.fee_payer;
    let back_deposit = get_sol_price(accounts.pyth_feed_account, AUCTION_DEPOSIT)?;
    invoke(&system_instruction::transfer(
        accounts.vault.key, accounts.fee_payer.key, back_deposit), 
        &[
            vault.clone(),
            fee_payer.clone(),
            accounts.system_program.clone(),
        ]
    )?;
    msg!("return deposit ok");

    // transfer to referrer
    let mut vault_percent = 100;
    let price = get_sol_price(accounts.pyth_feed_account, name_state.highest_price)?;
    
    invoke(&system_instruction::transfer(
        fee_payer.key, accounts.referrer_one.key, price * 40 / 100), 
        &[
            fee_payer.clone(),
            accounts.referrer_one.clone(),
            accounts.system_program.clone(),
        ]
    )?;
    vault_percent -= 40;
    msg!("transfer to referrer one ok");

    if let Some(refferrer_two) = accounts.referrer_two {
        invoke(&system_instruction::transfer(
            fee_payer.key, refferrer_two.key, price * 30 / 100), 
            &[
                fee_payer.clone(),
                refferrer_two.clone(),
                accounts.system_program.clone(),
            ]
        )?;
        vault_percent -= 30;
    }
    msg!("transfer to referrer two ok");

    if let Some(refferrer_three) = accounts.referrer_three {
        invoke(&system_instruction::transfer(
            fee_payer.key, refferrer_three.key, price * 20 / 100), 
            &[
                fee_payer.clone(),
                refferrer_three.clone(),
                accounts.system_program.clone(),
            ]
        )?;
        vault_percent -= 20;
    }
    msg!("transfer to referrer three ok");

    invoke(&system_instruction::transfer(
        fee_payer.key, accounts.state_rent_payer.key, price * 5 / 100), 
        &[
            fee_payer.clone(),
            accounts.state_rent_payer.clone(),
            accounts.system_program.clone(),
        ]
    )?;
    vault_percent -= 5;
    msg!("transfer to rent payer ok");

    invoke(&system_instruction::transfer(
        fee_payer.key, vault.key, price * vault_percent / 100), 
        &[
            fee_payer.clone(),
            vault.clone(),
            accounts.system_program.clone(),
        ]
    )?;
    msg!("transfer to vault ok");

    let central_state_signer_seeds: &[&[u8]] = &[&_program_id.to_bytes(), &[central_state::NONCE]];
    // cpi create domain
    let rent = Rent::from_account_info(accounts.rent_sysvar)?;

    Cpi::create_name_account(
        accounts.naming_service_program, 
        accounts.system_program, 
        accounts.name, 
        accounts.fee_payer, 
        accounts.root_domain, 
        accounts.central_state,
        hashed_name,
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
            params.name, 
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
