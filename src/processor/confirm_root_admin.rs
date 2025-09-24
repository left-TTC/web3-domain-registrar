use borsh::{BorshDeserialize, BorshSerialize};

use web3_utils::{
    accounts::InstructionsAccount,
    InstructionsAccount,
    borsh_size::BorshSize,
    BorshSize,
    check::check_account_owner
};
use solana_program::{
    msg, rent::Rent, sysvar::{self, Sysvar}
};
use web3_domain_name_service::{state::NameRecordHeader, utils::get_seeds_and_key};

use crate::{
    central_state, 
    constants::{SYSTEM_ID, WEB3_NAME_SERVICE}, 
    cpi::Cpi, 
    state::{RootStateRecordHeader}, 
    utils::{ get_hashed_name, CREATE_ROOT_TARGET}
};

use {
    web3_utils::{
        check::{check_account_key, check_signer},
    },
    
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
    },
};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
pub struct Params {
    pub root_name: String
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The naming service program ID
    pub naming_service_program: &'a T,
    /// The system program account
    pub system_program: &'a T,
    /// The fee payer account
    #[cons(writable, signer)]
    pub administrator: &'a T,
    /// The accoount to save fund state
    pub root_state_account: &'a T,
    /// The registrar central state account
    pub central_state: &'a T,
    /// root domain name account
    #[cons(writable)]
    pub root_name_account: &'a T,
    /// root domain's reverse lookup account
    #[cons(writable)]
    pub root_reverse_lookup: &'a T,
    /// The rent sysvar account
    pub rent_sysvar: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            naming_service_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            administrator: next_account_info(accounts_iter)?,
            root_state_account: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            root_name_account: next_account_info(accounts_iter)?,
            root_reverse_lookup: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
        };

        check_account_key(accounts.naming_service_program,  &WEB3_NAME_SERVICE)?;
        check_account_key(accounts.system_program, &SYSTEM_ID)?;
        check_account_key(accounts.central_state, &central_state::KEY)?;
        check_account_key(accounts.rent_sysvar, &sysvar::rent::ID)?;

        // Check owners
        check_account_owner(accounts.root_state_account, &crate::ID)?;
        check_account_owner(accounts.root_name_account, &SYSTEM_ID)?;
        check_account_owner(accounts.root_reverse_lookup, &SYSTEM_ID)?;

        // Check signer
        check_signer(accounts.administrator)?;

        Ok(accounts)
    }
}

pub fn process_confirm_root(
    _program_id: &Pubkey, 
    accounts: &[AccountInfo], 
    params: Params
) -> ProgramResult {
    msg!("root name: {:?}", params.root_name);

    let accounts = Accounts::parse(accounts)?;
    msg!("parse ok");

    let hashed_name_account = get_hashed_name(&params.root_name);

    let root_state_account = accounts.root_state_account;
    let (root_state_key, _) = get_seeds_and_key(
        &crate::ID, 
        hashed_name_account.clone(), 
        None, 
        None
    );
    if root_state_key != *root_state_account.key {
        msg!("The given root state account is incorrect.");
        return Err(ProgramError::InvalidArgument);
    }
    msg!("rootState ok");

    let root_state_account_data = root_state_account.data.borrow();
    let root_record_header = 
        RootStateRecordHeader::unpack_from_slice(&root_state_account_data)?;

    let root_name_account = accounts.root_name_account;
    let (root_name_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_name_account.clone(), 
        None, 
        None
    );
    check_account_key(root_name_account, &root_name_key)?;
    msg!("root_name_account ok");

    let hashed_reverse_lookup = get_hashed_name(&root_name_key.to_string());
    let root_reverse_account = accounts.root_reverse_lookup;
    let (reserse_look_up, _) = get_seeds_and_key(
        accounts.naming_service_program.key, 
        hashed_reverse_lookup.clone(), 
        Some(&central_state::KEY), 
        None
    );
    check_account_key(root_reverse_account, &reserse_look_up)?;
    msg!("root_reverse_lookup ok");

    let rent = Rent::from_account_info(accounts.rent_sysvar)?;
    let central_state_signer_seeds: &[&[u8]] = &[&crate::ID.to_bytes(), &[central_state::NONCE]];

    if root_record_header.amount >= CREATE_ROOT_TARGET {
        
        msg!("create root account");
        Cpi::create_root_name_account(
            accounts.naming_service_program,
            accounts.system_program,
            root_name_account,
            accounts.administrator,
            accounts.central_state,
            hashed_name_account,
            rent.minimum_balance(NameRecordHeader::LEN),
            // because this is the root domain
            // means we don't need the parent name owner's signature
        )?;

        msg!("create root reverse account");
        if root_reverse_account.data_len() == 0 {
            Cpi::create_reverse_lookup_account(accounts.naming_service_program, 
                accounts.system_program, 
                accounts.root_reverse_lookup, 
                accounts.administrator, 
                params.root_name, 
                hashed_reverse_lookup, 
                accounts.central_state, 
                
                accounts.rent_sysvar, 
                central_state_signer_seeds, 
                None, 
                None
            )?;
        }

    }else {
        msg!("not enough");
        return Err(ProgramError::InvalidArgument);
    }
    
    Ok(())
}
