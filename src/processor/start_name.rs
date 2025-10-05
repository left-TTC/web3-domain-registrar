//! Create a domain name and buy the ownership of a domain name

use std::fmt::format;

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
    state::{NameStateRecordHeader, RefferrerRecordHeader, ReverseLookup}, 
    utils::{check_state_time, get_hashed_name, get_now_time, get_sol_price, START_PRICE, TIME}
};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    pub root_name: String,
    pub price_decimals: u64,
    pub refferrer_key: Pubkey,
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
    /// The domain auction state reverse account
    #[cons(writable)]
    pub domain_state_reverse_account: &'a T,
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
    #[cons(writable)]
    pub refferrer_record_account: &'a T,
    /// vault
    #[cons(writable)]
    pub vault: &'a T,
    /// rent payer
    pub rent_payer: &'a T,
    /// refferrer's refferrer record account
    pub superior_refferrer_record: Option<&'a T>,
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
            domain_state_reverse_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            pyth_feed_account: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            refferrer_record_account: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            rent_payer: next_account_info(accounts_iter)?,
            superior_refferrer_record: next_account_info(accounts_iter).ok(),
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
        // second: register a already registered account =>
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
    msg!("name: {}", params.name);

    // the referreer record account
    let refferrer_record_account = accounts.refferrer_record_account;
    let (refferrer_record, refferrer_seeds) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&accounts.fee_payer.key.to_string()), 
        Some(&crate::ID), 
        Some(&crate::ID),
    );
    check_account_key(refferrer_record_account, &refferrer_record)?;
    msg!("payer's refferrer record account ok");

    let rent = Rent::from_account_info(accounts.rent_sysvar)?;

    let vault = accounts.vault;
    let (vault_key, _) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name("vault"), 
        Some(&central_state::KEY), 
        Some(&central_state::KEY)
    );
    check_account_key(vault, &vault_key)?;
    msg!("vault ok");

    if refferrer_record_account.data_len() == 0 {
        
        msg!("payer's refferrer record account need to be intialized");

        let refferrer_record_lamports = rent.minimum_balance(RefferrerRecordHeader::LEN);

        if params.refferrer_key != vault_key {
            msg!("payer use other's invitation");
            if let Some(superior_refferrer) = accounts.superior_refferrer_record {
                let (superior_refferrer_key, _) = get_seeds_and_key(
                    &crate::ID, 
                    get_hashed_name(&params.refferrer_key.to_string()), 
                    Some(&crate::ID), 
                    Some(&crate::ID)
                );
                check_account_key(superior_refferrer, &superior_refferrer_key)?;

                msg!("refferr's refferrer record account ok");
                
                let _state =  
                    RefferrerRecordHeader::unpack_from_slice(&superior_refferrer.data.borrow())?;
                msg!("refeerrer's refferrer is valid");
            } else {
                msg!("you should provide your refferrer's refferrer record"); 
                return Err(ProgramError::InvalidArgument);
            }
        }

        invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, &refferrer_record, refferrer_record_lamports), 
            &[
                accounts.fee_payer.clone(),
                accounts.refferrer_record_account.clone(),
                accounts.system_program.clone(),
            ],
        )?;

        invoke_signed(
            &system_instruction::allocate(
                &refferrer_record, 
                RefferrerRecordHeader::LEN as u64
            ), 
            &[accounts.refferrer_record_account.clone(), accounts.system_program.clone()], 
            &[&refferrer_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;

        invoke_signed(
            &system_instruction::assign(&refferrer_record, &crate::ID),
            &[accounts.refferrer_record_account.clone(), accounts.system_program.clone()],
            &[&refferrer_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )?;

        msg!("create payer's refferrer record account");
        let mut data = accounts.refferrer_record_account.data.borrow_mut();
        data[..32].copy_from_slice(&params.refferrer_key.to_bytes());

        msg!("write in refferrer record account");
    }else {
        let refferrer_data = 
            RefferrerRecordHeader::unpack_from_slice(&refferrer_record_account.data.borrow())?;
        if refferrer_data.refferrer_account != params.refferrer_key {
            msg!("regferrer is not unique");
            return Err(ProgramError::InvalidArgument);
        }
    }
    
    let (root_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key, 
        get_hashed_name(&params.root_name), 
        None, 
        None
    );
    check_account_key(accounts.root_domain, &root_account_key)?;
    msg!("root account ok");

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
    msg!("name state ok");

    let name_state_reverse = accounts.domain_state_reverse_account;
    let (name_state_reverse_key, name_state_reverse_seed) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&name_state_key.to_string()), 
        Some(&central_state::KEY), 
        None
    );
    check_account_key(name_state_reverse, &name_state_reverse_key)?;
    msg!("name state reverse ok");

    let domain_start_price = 
        get_sol_price(accounts.pyth_feed_account, params.price_decimals)?; 
    let deposit: u64;

    // initiate or start twice auction;
    if name_state_account.data_is_empty() {
        
        msg!("this domain is the frist time auction");
        deposit = domain_start_price / 10;
        msg!("start deposit: {:?}", deposit);

        let rent = Rent::from_account_info(accounts.rent_sysvar)?;
        let name_state_lamports = rent.minimum_balance(NameStateRecordHeader::LEN);

        if params.price_decimals != START_PRICE {
            msg!("error start price");
            return Err(ProgramError::InvalidArgument);
        }

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
            accounts.rent_payer.key,
            Clock::get()?.unix_timestamp, 
            params.price_decimals
        );
        first_name_state_record.pack_into_slice(& mut name_state_account.data.borrow_mut());
        msg!("write name state ok");

        let name_bytes = ReverseLookup { name: format!("{}.{}", params.name, params.root_name) }.try_to_vec().unwrap();
        msg!("reverse name {}", format!("{}.{}", params.name, params.root_name));
        invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, &name_state_reverse_key, name_state_lamports), 
            &[
                accounts.fee_payer.clone(),
                accounts.domain_state_reverse_account.clone(),
                accounts.system_program.clone(),
            ],
        )?;
        invoke_signed(
            &system_instruction::allocate(
                &name_state_reverse_key, 
                name_bytes.len() as u64
            ), 
            &[accounts.domain_state_reverse_account.clone(), accounts.system_program.clone()], 
            &[&name_state_reverse_seed.chunks(32).collect::<Vec<&[u8]>>()],
        )?;
        invoke_signed(
            &system_instruction::assign(&name_state_reverse_key, &crate::ID),
            &[accounts.domain_state_reverse_account.clone(), accounts.system_program.clone()],
            &[&name_state_reverse_seed.chunks(32).collect::<Vec<&[u8]>>()],
        )?;
        {
            let mut data = accounts.domain_state_reverse_account.try_borrow_mut_data()?;
            data[..name_bytes.len()].copy_from_slice(&name_bytes);
            msg!("write name state reverse ok");
        }
    } else {
        // Second or subsequent auctions
        msg!("Second or subsequent auctions");
        
        let name_state_data = 
            NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
        if check_state_time(name_state_data.update_time)? != TIME::RESALE {
            msg!("auctioning, can't initiate auction");
            return Err(ProgramError::InvalidArgument);
        }

        let rent_payer = accounts.rent_payer;
        if name_state_data.rent_payer != *rent_payer.key {
            msg!("fault rent payer account");
        }

        invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, rent_payer.key, domain_start_price * 3 / 100), 
            &[
                accounts.fee_payer.clone(),
                accounts.rent_payer.clone(),
                accounts.system_program.clone(),
            ]
        )?;
        msg!("transfer fee to rent payer: {:?}", domain_start_price * 3 / 100);

        // Initiate if there is no auction status
        let domain_account_data = 
            NameRecordHeader::unpack_from_slice(&accounts.domain_name_account.data.borrow());
        
        match domain_account_data {
            Ok(data) => {
                if params.price_decimals < data.custom_price {
                    msg!("poor");
                    return Err(ProgramError::InvalidArgument);
                }
                deposit = domain_start_price * 2 / 100;
                msg!("have already created: {}", deposit);
            }
            Err(_) => {
                msg!("this name account has been auctioned, but not created");
                deposit = domain_start_price * 7 / 100;
                if params.price_decimals != START_PRICE {
                    msg!("error start price");
                    return Err(ProgramError::InvalidArgument);
                }
            }
        }

        let new_record = NameStateRecordHeader::new(
            accounts.fee_payer.key, 
            rent_payer.key, 
            get_now_time()?, 
            params.price_decimals
        );
        NameStateRecordHeader::pack(new_record, &mut name_state_account.data.borrow_mut())?;
        msg!("update the name record ok");
    }

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
