


use web3_domain_name_service::{state::NameRecordHeader, utils::get_seeds_and_key};
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
    central_state, constants::{ADMIN, SYSTEM_ID,}, cpi::Cpi, utils::{get_hashed_name, PROJECT_START}
};




#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    start_domain: String,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,
    /// name service
    pub name_service: &'a T,
    /// The administrator account
    #[cons(writable, signer)]
    pub administrator: &'a T,   
    /// init the vault PDA
    #[cons(writable)]
    pub vault: &'a T,
    /// web3 name account
    #[cons(writable)]
    pub web3_name_account: &'a T,
    /// web3 name reverse
    #[cons(writable)]
    pub web3_name_reverse: &'a T,
    /// rent sysvar
    pub rent_sysvar: &'a T,
    /// central state
    pub central_state: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            name_service: next_account_info(accounts_iter)?,
            administrator: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            web3_name_account: next_account_info(accounts_iter)?,
            web3_name_reverse: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.system_program, &SYSTEM_ID)?;
        check_account_key(accounts.administrator, &ADMIN)?;
        check_account_key(accounts.central_state, &central_state::KEY)?;
        check_account_key(accounts.rent_sysvar, &sysvar::rent::ID)?;

        // Check owners
        check_account_owner(accounts.vault, &SYSTEM_ID)?;

        // Check signer
        check_signer(accounts.administrator)?;

        Ok(accounts)
    }
}

pub fn process_start_project(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;

    if params.start_domain != "web3" {
        msg!("start should be web3");
        return Err(ProgramError::InvalidArgument);
    }

    let name = accounts.web3_name_account;
    let name_reverse = accounts.web3_name_reverse;
    let hashed_web3 = get_hashed_name(&params.start_domain);

    let (name_key, _) = get_seeds_and_key(
        accounts.name_service.key, 
        hashed_web3.clone(), 
        None, 
        None
    );
    check_account_key(accounts.web3_name_account, &name_key)?;

    let hashed_reverse = get_hashed_name(&name_key.to_string());
    let (name_reverse_key, _) = get_seeds_and_key(
        accounts.name_service.key, 
        hashed_reverse.clone(), 
        Some(&central_state::KEY), 
        None
    );
    check_account_key(name_reverse, &name_reverse_key)?;

    let vault = accounts.vault;
    let (vault_key, vault_seeds) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name("vault"), 
        Some(&central_state::KEY), 
        Some(&central_state::KEY)
    );
    check_account_key(vault, &vault_key)?;
    msg!("check vault ok");

    invoke(
    &system_instruction::transfer(
        accounts.administrator.key, accounts.vault.key, PROJECT_START), 
        &[
            accounts.administrator.clone(),
            vault.clone(),
            accounts.system_program.clone(),
        ],
    )?;

    invoke_signed(
        &system_instruction::assign(&vault_key, &crate::ID),
        &[accounts.vault.clone(), accounts.system_program.clone()],
        &[&vault_seeds.chunks(32).collect::<Vec<&[u8]>>()],
    )?;

    let rent = Rent::from_account_info(accounts.rent_sysvar)?;

    Cpi::create_root_name_account(
        accounts.name_service, 
        accounts.system_program, 
        name, 
        accounts.administrator,
        accounts.central_state,
        hashed_web3,
        rent.minimum_balance(NameRecordHeader::LEN),
    )?;

    let central_state_signer_seeds: &[&[u8]] = &[&crate::ID.to_bytes(), &[central_state::NONCE]];
    Cpi::create_reverse_lookup_account(accounts.name_service, 
        accounts.system_program, 
        accounts.web3_name_reverse, 
        accounts.administrator, 
        params.start_domain, 
        hashed_reverse, 
        accounts.central_state, 
        
        accounts.rent_sysvar, 
        central_state_signer_seeds, 
        None, 
        None
    )?;

    Ok(())
}