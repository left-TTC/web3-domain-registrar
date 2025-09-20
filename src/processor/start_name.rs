//! Create a domain name and buy the ownership of a domain name

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

use crate::{central_state, constants::{SYSTEM_ID, WEB3_NAME_SERVICE}, 
    state::{write_data, NameStateRecordHeader, ReferrerRecordHeader}, 
    utils::{check_state_time, get_hashed_name, get_now_time, get_sol_price, AUCTION_DEPOSIT, START_PRICE, TIME}
};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    pub price_usd: u64,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> {
    /// The naming service program ID
    pub naming_service_program: &'a T,
    /// The root domain account       
    pub root_domain: &'a T,
    /// The name account
    pub domain_name_account: &'a T,
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
    /// the referrer -- must be unique
    /// one question: if usr want to get the profile from his next level
    /// he or she must keep the info is correspond
    pub referrer_account: &'a T,
    #[cons(writable)]
    pub referrer_record_account: &'a T,
    /// vault
    #[cons(writable)]
    pub vault: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            naming_service_program: next_account_info(accounts_iter)?,
            root_domain: next_account_info(accounts_iter)?,
            domain_name_account: next_account_info(accounts_iter)?,
            domain_state_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            pyth_feed_account: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            referrer_account: next_account_info(accounts_iter)?,
            referrer_record_account: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
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

        // when check account owner -> we have two direction:
        // frist: register a initial domain name => 
        //          domain state --> sys
        //          domain name --> sys
        //  second: register a already registered account =>
        //          domain state --> register
        //          domain name --> name service

        check_account_owner(self.root_domain, &WEB3_NAME_SERVICE)?;
        msg!("root_domain owner ok");

        // Check signer
        check_signer(self.fee_payer).unwrap();
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
    
    if params.name != params.name.trim().to_lowercase() {
        msg!("Domain names must be lower case and have no space");
        return Err(ProgramError::InvalidArgument);
    }
    if params.name.contains('.') {
        msg!("domain contains invalid puncation");
        return Err(ProgramError::InvalidArgument);
    }
    if params.price_usd < START_PRICE {
        msg!("invalid bidding");
        return Err(ProgramError::InvalidArgument);
    }
    msg!("name: {}", params.name);

    // the referreer record account
    let referrer_record_account = accounts.referrer_record_account;
    let (referrer_record, referrer_seeds) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&accounts.fee_payer.clone().key.to_string()), 
        Some(&crate::ID), 
        Some(&crate::ID),
    );
    check_account_key(referrer_record_account, &referrer_record)?;

    let rent = Rent::from_account_info(accounts.rent_sysvar)?;

    if referrer_record_account.data_len() == 0 {
        // usr init 
        let referrer_record_lamports = rent.minimum_balance(ReferrerRecordHeader::LEN);

        invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, &referrer_record, referrer_record_lamports), 
            &[
                accounts.fee_payer.clone(),
                accounts.referrer_record_account.clone(),
                accounts.system_program.clone(),
            ],
        )?;

        invoke_signed(
            &system_instruction::allocate(
                &referrer_record, 
                ReferrerRecordHeader::LEN as u64
            ), 
            &[accounts.referrer_record_account.clone(), accounts.system_program.clone()], 
            &[&referrer_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;

        invoke_signed(
            &system_instruction::assign(&referrer_record, &crate::ID),
            &[accounts.referrer_record_account.clone(), accounts.system_program.clone()],
            &[&referrer_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;
    }else {
        let referrer_data = 
            ReferrerRecordHeader::unpack_from_slice(&referrer_record_account.data.borrow())?;
        if &referrer_data.referrer_account != accounts.referrer_account.key {
            msg!("regferrer is not unique");
            return Err(ProgramError::InvalidArgument);
        }
    }

    let (name_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key, 
        get_hashed_name(&params.name), 
        None, 
        Some(accounts.root_domain.key)
    );
    check_account_key(accounts.domain_name_account, &name_account_key)?;
    msg!("name account ok");

    let name_state_account = accounts.domain_state_account;
    let (name_state_key, name_state_seeds) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&params.name), 
        Some(&central_state::KEY), 
        Some(accounts.root_domain.key)
    );
    check_account_key(name_state_account, &name_state_key)?;

    let vault = accounts.vault;
    let (vault_key, _) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name("vault"), 
        Some(&central_state::KEY), 
        Some(&central_state::KEY)
    );
    check_account_key(vault, &vault_key)?;

    // initiate or start twice auction;
    if name_state_account.data_is_empty() {
        //initiate
        let rent = Rent::from_account_info(accounts.rent_sysvar)?;
        let name_state_lamports = rent.minimum_balance(NameStateRecordHeader::LEN);

        invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, &name_state_key, name_state_lamports), 
            &[
                accounts.fee_payer.clone(),
                accounts.domain_state_account.clone(),
                accounts.system_program.clone(),
            ],
        )?;
        invoke_signed(
            &system_instruction::allocate(
                &name_state_key, 
                NameStateRecordHeader::LEN as u64
            ), 
            &[name_state_account.clone(), accounts.system_program.clone()], 
            &[&name_state_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;
        invoke_signed(
            &system_instruction::assign(&name_state_key, &crate::ID),
            &[name_state_account.clone(), accounts.system_program.clone()],
            &[&name_state_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;

        let first_name_state_record = NameStateRecordHeader::new(
            accounts.fee_payer.key, 
            accounts.fee_payer.key, 
            Clock::get()?.unix_timestamp, 
            params.price_usd
        );
        first_name_state_record.pack_into_slice(& mut name_state_account.data.borrow_mut());
    } else {
        // Second or subsequent auctions
        let name_state_data = 
            NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
        if check_state_time(name_state_data.update_time)? != TIME::RESALE {
            msg!("auctioning, can't initiate auction");
            return Err(ProgramError::InvalidArgument);
        }

        // Initiate if there is no auction status
        let domain_account_data = 
            NameRecordHeader::unpack_from_slice(&accounts.domain_name_account.data.borrow())?;
        if params.price_usd < domain_account_data.custom_price {
            msg!("poor");
            return Err(ProgramError::InvalidArgument);
        }

        let highest_bidder = accounts.fee_payer.key.to_bytes();
        write_data(name_state_account, &highest_bidder, 0);
        let start_time = get_now_time()?.to_le_bytes();
        write_data(name_state_account, &start_time, 64);
        let highest_price = params.price_usd.to_le_bytes();
        write_data(name_state_account, &highest_price, 72);
    }

    let deposit = 
    get_sol_price(accounts.pyth_feed_account, AUCTION_DEPOSIT)?; 
    invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, &vault_key, deposit), 
            &[
                accounts.fee_payer.clone(),
                accounts.vault.clone(),
                accounts.system_program.clone(),
            ]
    )?;
    msg!("transfer deposit to vault");

    Ok(())
}
