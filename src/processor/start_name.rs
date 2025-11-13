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

use crate::{central_state, constants::{SYSTEM_ID, return_vault_key}, 
    state::{NameStateRecordHeader, ReferrerRecordHeader, ReverseLookup, get_referrer_record_key}, 
    utils::{TIME, check_state_time, get_hashed_name, get_now_time, if_referrer_valid}
};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    pub root_name: String,
    pub price_sol: u64,
    pub referrer_key: Pubkey,
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
    /// The rent sysvar account
    pub rent_sysvar: &'a T,
    #[cons(writable)]
    pub referrer_record_account: &'a T,
    /// vault
    #[cons(writable)]
    pub vault: &'a T,
    /// referrer's referrer record account
    pub superior_referrer_record: Option<&'a T>,
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
            rent_sysvar: next_account_info(accounts_iter)?,
            referrer_record_account: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            superior_referrer_record: next_account_info(accounts_iter).ok(),
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_key(self.naming_service_program, &web3_domain_name_service::ID).unwrap();
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

        check_account_owner(self.root_domain, &web3_domain_name_service::ID)?;
        msg!("root_domain owner ok");

        // Check signer
        check_signer(self.fee_payer).unwrap();
        msg!("fee_payer signature ok");

        Ok(())
    }
}


// trnasfer all
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

    let rent = Rent::from_account_info(accounts.rent_sysvar)?;

    let vault = accounts.vault;
    let (vault_key, _) = return_vault_key();

    check_account_key(vault, &vault_key)?;
    msg!("vault ok");

    // the referreer record account
    let referrer_record_account = accounts.referrer_record_account;
    let (referrer_record, referrer_seeds) = 
        get_referrer_record_key(&accounts.fee_payer.key);
    check_account_key(referrer_record_account, &referrer_record)?;

    msg!("payer's referrer record account ok");

    if referrer_record_account.data_len() == 0 {
        
        msg!("payer's referrer record account need to be intialized");
        let referrer_record_lamports = rent.minimum_balance(ReferrerRecordHeader::LEN);

        if params.referrer_key != vault_key {
            msg!("payer uses other's invitation code");
            if let Some(superior_referrer) = accounts.superior_referrer_record {
                let (superior_referrer_key, _) = get_referrer_record_key(&params.referrer_key);
                check_account_key(superior_referrer, &superior_referrer_key)?;

                msg!("refferr's referrer record account ok");
                
                let referrer_referrer_state =  
                    ReferrerRecordHeader::unpack_from_slice(&superior_referrer.data.borrow())?;

                if !if_referrer_valid(referrer_referrer_state)?{
                    return Err(ProgramError::InvalidArgument);
                }
            } else {
                msg!("you should provide your referrer's referrer record"); 
                return Err(ProgramError::InvalidArgument);
            }
        }

        invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, &referrer_record, referrer_record_lamports), 
            &[
                accounts.fee_payer.clone(),
                accounts.referrer_record_account.clone(),
                accounts.system_program.clone(),
            ],
        )?;
        msg!("referrer record: {:?}", referrer_record_lamports);

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

        msg!("create payer's referrer record account");

        let record = ReferrerRecordHeader::new(
            params.referrer_key,
            get_now_time()?,
        );

        let mut data = accounts.referrer_record_account.try_borrow_mut_data()?;
        record.pack_into_slice(&mut data);

        msg!("write in referrer record account");

    }else {
        let referrer_data = 
            ReferrerRecordHeader::unpack_from_slice(&referrer_record_account.data.borrow())?;
        if referrer_data.referrer_account != params.referrer_key {
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

    // initiate or start twice auction;
    if name_state_account.data_is_empty() {
        
        msg!("this domain is the frist time auction");
        if params.price_sol != 10_000_000 {
            msg!("error start price");
            return Err(ProgramError::InvalidArgument);
        }

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
            Clock::get()?.unix_timestamp, 
            params.price_sol
        );
        first_name_state_record.pack_into_slice(& mut name_state_account.data.borrow_mut());
        msg!("write name state ok");

        let name_bytes = ReverseLookup { name: format!("{}.{}", params.name, params.root_name) }.try_to_vec().unwrap();
        msg!("reverse name {}", format!("{}.{}", params.name, params.root_name));

        // create reverse key and write domain data
        invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, &name_state_reverse_key, rent.minimum_balance(name_bytes.len() as usize)), 
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

        msg!("name state: {:?}", rent.minimum_balance(name_bytes.len() as usize) + name_state_lamports);
    } else {
        // Second or subsequent auctions
        msg!("Second or subsequent auctions");
        
        let name_state_data = 
            NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
        if check_state_time(name_state_data.update_time)? != TIME::PENDING {
            msg!("auctioning, can't initiate auction");
            return Err(ProgramError::InvalidArgument);
        }

        // Initiate if there is no auction status
        let domain_account_data = 
            NameRecordHeader::unpack_from_slice(&accounts.domain_name_account.data.borrow())
            .map_err(|e| {
                msg!("this name account has been auctioned, but not created");
                msg!("waiting for confirm");
                e
            })?;

        if domain_account_data.owner != name_state_data.highest_bidder {
            msg!("this domain hasn't been confirm");
            return Err(ProgramError::InvalidArgument);
        }

        if params.price_sol != domain_account_data.custom_price {
            msg!("error value");
            return Err(ProgramError::InvalidArgument);
        }
        if accounts.fee_payer.key == &domain_account_data.owner {
            msg!("you can't start your own domain");
            return Err(ProgramError::InvalidArgument);
        }
        
        let new_record = NameStateRecordHeader::new(
            accounts.fee_payer.key, 
            get_now_time()?, 
            params.price_sol
        );
        NameStateRecordHeader::pack(new_record, &mut name_state_account.data.borrow_mut())?;
        msg!("update the name record ok");
    }

    invoke(
        &system_instruction::transfer(
            accounts.fee_payer.key, &vault_key, params.price_sol), 
            &[
                accounts.fee_payer.clone(),
                accounts.vault.clone(),
                accounts.system_program.clone(),
            ]
    )?;
    msg!("transfer all to vault: {:?} sol", params.price_sol);

    Ok(())
}
