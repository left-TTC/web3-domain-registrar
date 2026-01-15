use borsh::{BorshDeserialize, BorshSerialize};

use web3_utils::{
    accounts::InstructionsAccount,
    InstructionsAccount,
    borsh_size::BorshSize,
    BorshSize,
    check::check_account_owner
};
use solana_program::{
    msg, rent::Rent, sysvar::Sysvar,
};
use web3_domain_name_service::{state::NameRecordHeader, utils::get_seeds_and_key};

use crate::{
    central_state, constants::{SYSTEM_ID, return_vault_key}, cpi::Cpi, state::{ReverseLookup, RootStateRecordHeader, write_data}, utils::{ CREATE_ROOT_TARGET, get_hashed_name}
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
    pub add_sol: u64,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub name_service: &'a T,
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
            name_service: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            root_state_account: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            root_name_account: next_account_info(accounts_iter)?,
            root_reverse_lookup: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
        };

        check_account_key(accounts.name_service, &web3_domain_name_service::ID)?;
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

    msg!("root name: {:?}, add: {:?} lamports", params.root_name, params.add_sol);

    let accounts = Accounts::parse(accounts)?;
    msg!("parse ok");

    let (vault, _) = return_vault_key();
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

    let mut added_amount = params.add_sol;
    {
        let root_state_account_data = root_state_account.data.borrow();
        let root_record_header = 
            RootStateRecordHeader::unpack_from_slice(&root_state_account_data)?;

        if root_record_header.amount >= CREATE_ROOT_TARGET {
            msg!("already enough");
            return Err(ProgramError::InvalidArgument);
        }

        added_amount += root_record_header.amount;
        
        msg!("used to be: {:?} and now {:?} lamports, add amount ok", root_record_header.amount, added_amount);
    }

    let bytes = added_amount.to_le_bytes();
    write_data(accounts.root_state_account, &bytes, 32);
    msg!("write amount ok");

    let mut difference: u64 = 0;

    if added_amount > CREATE_ROOT_TARGET {
        difference = added_amount - CREATE_ROOT_TARGET;

        let root_name_account = accounts.root_name_account;
        let (root_name_key, _) = get_seeds_and_key(
            accounts.name_service.key,
            hashed_name_account.clone(), 
            None, 
            None
        );
        check_account_key(root_name_account, &root_name_key)?;
        msg!("root_name_account ok");

        let hashed_reverse_lookup = get_hashed_name(&root_name_key.to_string());
        let root_reverse_account = accounts.root_reverse_lookup;
        let (reserse_look_up, _) = get_seeds_and_key(
            accounts.name_service.key, 
            hashed_reverse_lookup.clone(), 
            Some(&central_state::KEY), 
            None
        );
        check_account_key(root_reverse_account, &reserse_look_up)?;
        msg!("root_reverse_lookup ok");

        let rent = Rent::from_account_info(accounts.rent_sysvar)?;
        let central_state_signer_seeds: &[&[u8]] = &[&crate::ID.to_bytes(), &[central_state::NONCE]];

        let root_name_lamports = rent.minimum_balance(NameRecordHeader::LEN);

        msg!("create root account");
        Cpi::create_root_name_account(
            accounts.name_service,
            accounts.system_program,
            root_name_account,
            accounts.fee_payer,
            accounts.central_state,
            hashed_name_account,
            root_name_lamports,
        )?;

        msg!("create root reverse account");
        if root_reverse_account.data_len() == 0 {
            Cpi::create_reverse_lookup_account(accounts.name_service, 
                accounts.system_program, 
                accounts.root_reverse_lookup, 
                accounts.fee_payer, 
                params.root_name.clone(), 
                hashed_reverse_lookup, 
                accounts.central_state, 
                accounts.rent_sysvar, 
                central_state_signer_seeds, 
                None, 
                None
            )?;
        }

        let lamports = rent.minimum_balance(ReverseLookup { name: params.root_name }.try_to_vec().unwrap().len() + NameRecordHeader::LEN) + root_name_lamports;

        **accounts.vault.try_borrow_mut_lamports()? -= lamports;
        **accounts.fee_payer.try_borrow_mut_lamports()? += lamports;
    }

    invoke(
    &instruction::transfer(
            accounts.fee_payer.key,
            accounts.vault.key,
            params.add_sol - difference
        ), 
        &[
            accounts.fee_payer.clone(),
            accounts.vault.clone(),
            accounts.system_program.clone(),
        ],
    )?;
    
    Ok(())
}
