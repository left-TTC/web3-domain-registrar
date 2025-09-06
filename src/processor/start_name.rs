//! Create a domain name and buy the ownership of a domain name

use web3_name_service_utils::{
    checks::{check_account_key, check_account_owner, check_signer},
    BorshSize, InstructionsAccount,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_program, sysvar,
    sysvar::Sysvar,
    system_instruction
};
use web3_domain_name_service::state::NameRecordHeader;

use crate::{central_state, constants::WEB3_NAME_SERVICE, state::{NameStateRecordHeader, ReferrerRecordHeader}, utils::{check_state_time_valid, get_hashed_name, get_seeds_and_key}};


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
    /// The vault account     
    #[cons(writable)]
    pub vault: &'a T,
    /// The rent sysvar account
    pub rent_sysvar: &'a T,
    /// the referrer -- must be unique
    /// one question: if usr want to get the profile from his next level
    /// he or she must keep the info is correspond
    #[cons(writable)]
    pub referrer_account: &'a T,
    #[cons(writable)]
    pub referrer_record_account: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
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
            vault: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            referrer_account: next_account_info(accounts_iter)?,
            referrer_record_account: next_account_info(accounts_iter)?,
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_key(self.naming_service_program, &WEB3_NAME_SERVICE).unwrap();
        msg!("nameservice id ok");
        check_account_key(self.system_program, &system_program::ID).unwrap();
        msg!("system_program id ok");
        check_account_key(self.central_state, &central_state::KEY).unwrap();
        msg!("central_state id ok");
        check_account_key(self.rent_sysvar, &sysvar::rent::ID).unwrap();
        msg!("rent_sysvar id ok");

        // Check ownership
        check_account_owner(self.domain_name_account, &system_program::ID)
            .map_err(|_| crate::Error::AlreadyRegistered)?;
        check_account_owner(self.domain_state_account, &system_program::ID)
            .map_err(|_| crate::Error::AlreadyRegistered)?;
        check_account_owner(self.root_domain, &WEB3_NAME_SERVICE)?;
        msg!("root_domain owner ok");

        // Check signer
        check_signer(self.fee_payer).unwrap();
        msg!("fee_payer signature ok");

        Ok(())
    }
}

pub fn process_create(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    create(program_id, accounts, params)
}

pub fn create<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: Accounts<'a, AccountInfo<'b>>,
    params: Params,
) -> ProgramResult {
    
    accounts.check()?;
    
    if params.name != params.name.trim().to_lowercase() {
        msg!("Domain names must be lower case and have no space");
        return Err(ProgramError::InvalidArgument);
    }
    
    if params.name.contains('.') {
        return Err(ProgramError::InvalidArgument);
    }

    msg!("name: {}", params.name);

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
    if &name_account_key != accounts.domain_name_account.key {
        msg!("Provided wrong name account");
        return Err(ProgramError::InvalidArgument);
    }
    msg!("name account ok");

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
    
    // check if second auction
    if name_state_account.data_len() > 0 {
        let name_state_data = 
            NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
        if !check_state_time_valid(name_state_data.update_time)? {
            msg!("auctioning");
            return Err(ProgramError::InvalidArgument);
        }

        // now means second auction
        let domain_account_data = 
            NameRecordHeader::unpack_from_slice(&accounts.domain_name_account.data.borrow())?;
        if params.price_usd < domain_account_data.custom_price {
            msg!("poor");
            return Err(ProgramError::InvalidArgument);
        }
    } 

    if name_state_account.data_is_empty() {
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
            &[accounts.domain_state_account.clone(), accounts.system_program.clone()], 
            &[&name_state_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;

        invoke_signed(
            &system_instruction::assign(&name_state_key, &crate::ID),
            &[accounts.domain_state_account.clone(), accounts.system_program.clone()],
            &[&name_state_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;
    }


    Ok(())
}
