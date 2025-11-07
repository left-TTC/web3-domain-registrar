
use web3_domain_name_service::utils::get_seeds_and_key;
use web3_utils::{
    check::{check_account_owner, check_account_key, check_signer},
    BorshSize,
    borsh_size::BorshSize,
    InstructionsAccount,
    accounts::InstructionsAccount,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    msg,
    program::{invoke, invoke_signed},
    rent::Rent,
    sysvar::Sysvar,
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar,
};
use solana_system_interface::instruction as system_instruction;
use crate::{
    central_state, constants::{SYSTEM_ID}, utils::{get_hashed_name, ADVANCED_STORAGE}
};

use crate::state::RootStateRecordHeader;



#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    pub root_name: String,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,
    /// The fee payer account
    #[cons(writable, signer)]
    pub initiator: &'a T,
    #[cons(writable)]
    pub root_state_account: &'a T,
    /// root domain name account
    pub root_name_account: &'a T,
    /// The vault account     
    #[cons(writable)]
    pub vault: &'a T,
    /// The rent sysvar account
    pub rent_sysvar: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            initiator: next_account_info(accounts_iter)?,
            root_state_account: next_account_info(accounts_iter)?,
            root_name_account: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &SYSTEM_ID)?;
        check_account_key(accounts.rent_sysvar, &sysvar::rent::ID)?;

        msg!("lamports: {:?}", accounts.root_state_account.lamports());
        msg!("owner: {:?}", accounts.root_state_account.owner);
        check_account_owner(accounts.root_state_account, &SYSTEM_ID)?;

        // Check signer
        check_signer(accounts.initiator)?;

        Ok(accounts)
    }
}

pub fn process_initiate_root(
    _program_id: &Pubkey,
     accounts: &[AccountInfo],
      params: Params
) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;

    let root_state_account = accounts.root_state_account;
    let (root_state_key, seeds) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&params.root_name), 
        None, 
        None
    );
    if root_state_key != *root_state_account.key {
        msg!("The given root state account is incorrect.");
        return Err(ProgramError::InvalidArgument);
    }

    let (root_name_account, _) = get_seeds_and_key(
        &web3_domain_name_service::ID, 
        get_hashed_name(&params.root_name), 
        None, 
        None,
    );
    check_account_key(accounts.root_name_account, &root_name_account)?;

    let (vault, _) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name("vault"), 
        Some(&central_state::KEY), 
        Some(&central_state::KEY)
    );
    check_account_key(accounts.vault, &vault)?;
    msg!("check vault ok");

    if root_state_account.data.borrow().len() > 0 {
        msg!("the root state account's length > 0");
        let _root_record_header = 
            RootStateRecordHeader::unpack_from_slice(&root_state_account.data.borrow())?;
    }
    msg!("root state account ok");

    let mut extra_lamports = ADVANCED_STORAGE;

    // if the root state account doesn't created
    if root_state_account.data.borrow().len() == 0 {

        let rent = Rent::from_account_info(accounts.rent_sysvar)?;
        let root_state_lamports = rent.minimum_balance(RootStateRecordHeader::LEN);
        
        invoke(
        &system_instruction::transfer(
            accounts.initiator.key, &root_state_key, root_state_lamports), 
            &[
                accounts.initiator.clone(),
                accounts.root_state_account.clone(),
                accounts.system_program.clone(),
            ],
        )?;

        invoke_signed(
            &system_instruction::allocate(
                &root_state_key, 
                RootStateRecordHeader::LEN as u64
            ), 
            &[accounts.root_state_account.clone(), accounts.system_program.clone()], 
            &[&seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;

        invoke_signed(
            &system_instruction::assign(&root_state_key, &crate::ID),
            &[accounts.root_state_account.clone(), accounts.system_program.clone()],
            &[&seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;

        extra_lamports -= root_state_lamports;
    }else {
        msg!("root state length err"); 
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    msg!("crea root state account ok");

    invoke(
    &system_instruction::transfer(
        accounts.initiator.key, accounts.vault.key, extra_lamports), 
        &[
            accounts.initiator.clone(),
            accounts.vault.clone(),
            accounts.system_program.clone(),
        ],
    )?;
    msg!("transfer to vault ok");


    let init_state: RootStateRecordHeader = RootStateRecordHeader::
        new(
            *accounts.initiator.key, 
            ADVANCED_STORAGE, 
            &params.root_name
        );
    
    init_state.pack_into_slice(&mut accounts.root_state_account.data.borrow_mut());
    msg!("write root state data ok");

    Ok(())
}