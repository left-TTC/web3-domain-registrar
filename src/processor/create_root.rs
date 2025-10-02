use borsh::{BorshDeserialize, BorshSerialize};

use web3_utils::{
    accounts::InstructionsAccount,
    InstructionsAccount,
    borsh_size::BorshSize,
    BorshSize,
    check::check_account_owner
};
use solana_program::{
    msg,
};
use web3_domain_name_service::{utils::get_seeds_and_key};

use crate::{
    central_state, 
    constants::{SYSTEM_ID, WEB3_NAME_SERVICE}, 
    state::{write_data, RootStateRecordHeader}, 
    utils::{ get_hashed_name, get_sol_price, CREATE_ROOT_TARGET}
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
        program::invoke,
    },
};

use solana_system_interface::instruction as instruction;


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
pub struct Params {
    pub root_name: String,
    pub add: u64,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
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
    pub pyth_feed_account: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            root_state_account: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            pyth_feed_account: next_account_info(accounts_iter)?,
        };

        check_account_key(accounts.system_program, &SYSTEM_ID)?;
        check_account_key(accounts.central_state, &central_state::KEY)?;

        // Check owners
        check_account_owner(accounts.root_state_account, &crate::ID)?;
        check_account_owner(accounts.vault, &crate::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

// only add the rooy state
pub fn process_create_root(
    _program_id: &Pubkey, 
    accounts: &[AccountInfo], 
    params: Params
) -> ProgramResult {

    msg!("root name: {:?}, add: {:?}", params.root_name, params.add);

    let accounts = Accounts::parse(accounts)?;
    msg!("parse ok");

    let (vault, _) = get_seeds_and_key(
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
    if root_state_key != *root_state_account.key {
        msg!("The given root state account is incorrect.");
        return Err(ProgramError::InvalidArgument);
    }
    msg!("rootState ok");

    let mut added_amount = params.add;
    {
        let root_state_account_data = root_state_account.data.borrow();
        let root_record_header = 
            RootStateRecordHeader::unpack_from_slice(&root_state_account_data)?;

        if root_record_header.amount >= CREATE_ROOT_TARGET {
            msg!("already enough");
            return Err(ProgramError::InvalidArgument);
        }

        added_amount += root_record_header.amount;
        
        msg!("used to be: {:?} and now {:?}, add amount ok", root_record_header.amount, added_amount);
    }

    let bytes = added_amount.to_le_bytes();
    write_data(accounts.root_state_account, &bytes, 32);
    msg!("write amount ok");

    let mut difference: u64 = 0;

    if added_amount > CREATE_ROOT_TARGET {
        difference = added_amount - CREATE_ROOT_TARGET;
    }

    let add_token_price = 
        get_sol_price(&accounts.pyth_feed_account, params.add - difference)?;
    msg!("get add token price: {:?}", add_token_price );

    invoke(
    &instruction::transfer(
            accounts.fee_payer.key,
            accounts.vault.key,
            add_token_price
        ), 
        &[
            accounts.fee_payer.clone(),
            accounts.vault.clone(),
            accounts.system_program.clone(),
        ],
    )?;
    
    Ok(())
}
