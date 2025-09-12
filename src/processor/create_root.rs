use borsh::{BorshDeserialize, BorshSerialize};

use web3_utils::{
    accounts::InstructionsAccount,
    InstructionsAccount,
    borsh_size::BorshSize,
    BorshSize,
};
use web3_utils::{check::check_account_owner};
use solana_program::{
    msg,
    rent::Rent,
    sysvar,
    sysvar::Sysvar,
};
use web3_domain_name_service::state::NameRecordHeader;

use crate::{
    central_state, 
    constants::WEB3_NAME_SERVICE, 
    cpi::Cpi, 
    state::{write_data, RootStateRecordHeader}, 
    utils::{ get_hashed_name, get_seeds_and_key, get_sol_price, CREATE_ROOT_TARGET}
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
        system_program,
        program::invoke,
    },
};




use crate::cpi;


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
pub struct Params {
    pub root_name: String,
    pub add: u64,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The naming service program ID
    pub naming_service_program: &'a T,
    /// The system program account
    pub system_program: &'a T,
    /// The vault account     
    #[cons(writable)]
    pub vault: &'a T,
    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    /// The accoount to save fund state
    #[cons(writable)]
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
    /// The Pyth feed account
    pub pyth_feed_account: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            naming_service_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            root_state_account: next_account_info(accounts_iter)?,
            root_name_account: next_account_info(accounts_iter)?,
            root_reverse_lookup: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            pyth_feed_account: next_account_info(accounts_iter)?,
        };

        check_account_key(accounts.naming_service_program,  &WEB3_NAME_SERVICE)?;
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.central_state, &central_state::KEY)?;
        check_account_key(accounts.rent_sysvar, &sysvar::rent::ID)?;

        // Check owners
        check_account_owner(accounts.root_state_account, &crate::ID)?;
        check_account_owner(accounts.root_name_account, &system_program::ID)?;
        check_account_owner(accounts.root_reverse_lookup, &system_program::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}


pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    if params.add < 1000 {
        msg!("add amount is too small");
        return Err(ProgramError::InvalidArgument);
    }
    let accounts = Accounts::parse(accounts)?;
    msg!("parse ok");

    let (vault, vault_seed) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name("vault"), 
        Some(&central_state::KEY), 
        Some(&central_state::KEY)
    );
    check_account_key(accounts.vault, &vault)?;
    msg!("check vault ok");

    let hashed_name_account = get_hashed_name(&params.root_name);
    let root_state_account = accounts.root_state_account;

    let (root_state_key, _) = get_seeds_and_key(
        &crate::ID, 
        hashed_name_account.clone(), 
        None, 
        None
    );
    check_account_key(root_state_account, &root_state_key)?;
    msg!("rootState ok");

    let root_state_account_data = root_state_account.data.borrow();
    if root_state_account_data.len() != 72 {
        msg!("root state account's length error");
        return  Err(ProgramError::InvalidArgument);
    }
    
    let root_record_header = 
        RootStateRecordHeader::unpack_from_slice(&root_state_account_data)?;

    if root_record_header.get_name() != params.root_name{
        msg!("fault root name");
        return Err(ProgramError::InvalidArgument);
    }

    let added_amount = root_record_header.amount + params.add;
    msg!("used to be: {:?} and now {:?}, add amount ok", root_record_header.amount, added_amount);

    let bytes = added_amount.to_le_bytes();
    msg!("added_amount bytes: {:?}", bytes);
    write_data(accounts.root_state_account, &bytes, 32);
    msg!("write ok");

    let mut difference: u64 = 0;

    if added_amount >= CREATE_ROOT_TARGET {
        let (root_name_key, _) = get_seeds_and_key(
            accounts.naming_service_program.key,
            hashed_name_account.clone(), 
            None, 
            None
        );
        check_account_key(accounts.root_name_account, &root_name_key)?;
        msg!("root_name_account ok");

        let hashed_reverse_lookup = get_hashed_name(&root_name_key.to_string());
        let (reserse_look_up, _) = get_seeds_and_key(
            accounts.naming_service_program.key, 
            hashed_reverse_lookup.clone(), 
            Some(&central_state::KEY), 
            None
        );
        check_account_key(accounts.root_reverse_lookup, &reserse_look_up)?;
        msg!("root_reverse_lookup ok");

        let rent = Rent::from_account_info(accounts.rent_sysvar)?;
        let central_state_signer_seeds: &[&[u8]] = &[&crate::ID.to_bytes(), &[central_state::NONCE]];

        msg!("create root account");
        Cpi::create_root_name_account(
            accounts.naming_service_program,
            accounts.system_program,
            accounts.root_name_account,
            accounts.vault,
            accounts.central_state,
            hashed_name_account,
            rent.minimum_balance(NameRecordHeader::LEN),
            // because this is the root domain
            // means we don't need the parent name owner's signature
            &vault_seed,
        )?;

        msg!("create root reverse account");
        if accounts.root_reverse_lookup.data_len() == 0 {
            Cpi::create_root_reverse_lookup_account(
                accounts.naming_service_program, 
                accounts.system_program, 
                accounts.root_reverse_lookup, 
                accounts.vault, 
                params.root_name, 
                hashed_reverse_lookup, 
                accounts.central_state, 
                accounts.rent_sysvar, 
                central_state_signer_seeds, 
                &vault_seed,
            )?;
        }

        difference = added_amount - CREATE_ROOT_TARGET;
    }

    let add_token_price = 
        get_sol_price(&accounts.pyth_feed_account, params.add - difference)?;
    msg!("get add token price: {:?}", add_token_price );

    invoke(
    &system_instruction::transfer(
        accounts.fee_payer.key, &root_state_key, add_token_price), 
        &[
            accounts.fee_payer.clone(),
            accounts.root_state_account.clone(),
            accounts.system_program.clone(),
        ],
    )?;
    
    Ok(())
}
