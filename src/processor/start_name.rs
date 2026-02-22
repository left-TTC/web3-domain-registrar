//! Create a domain name and buy the ownership of a domain name

use web3_utils::{
    check::{check_account_key, check_account_owner, check_signer},
    BorshSize, InstructionsAccount,
    borsh_size::BorshSize,
    accounts::InstructionsAccount,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, clock::Clock, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent, sysvar::Sysvar
};
use web3_domain_name_service::{state::NameRecordHeader, utils::get_seeds_and_key};
use solana_system_interface::instruction as system_instruction;

use crate::{central_state, constants::return_vault_key, cpi::Cpi, state::{NameStateRecordHeader, ReferrerRecordHeader, get_referrer_record_key}, utils::{get_hashed_name, get_now_time, if_referrer_valid, math}
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
pub struct Accounts<'a, T> {
    /// The naming service program ID
    pub naming_service_program: &'a T,

    /// The root domain account       
    pub root_domain: &'a T,

    /// The name account
    #[cons(writable)]
    pub domain_name_account: &'a T,

    /// The reverse look up account   
    #[cons(writable)]
    pub reverse_lookup: &'a T,

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

    #[cons(writable)]
    pub referrer_record_account: &'a T,

    /// vault
    #[cons(writable)]
    pub vault: &'a T,

    /// last owner -- could be default
    #[cons(writable)]
    pub last_owner: &'a T,

    /// rent sysvar
    pub rent_sysvar: &'a T,

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
            reverse_lookup: next_account_info(accounts_iter)?,
            domain_state_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            referrer_record_account: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            last_owner: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            superior_referrer_record: next_account_info(accounts_iter).ok(),
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_key(self.naming_service_program, &web3_domain_name_service::ID)?;
        msg!("nameservice id ok");
        check_account_key(self.system_program, &solana_program::system_program::ID)?;
        msg!("system_program id ok");
        check_account_key(self.central_state, &central_state::KEY)?;
        msg!("central_state id ok");

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
        check_signer(self.fee_payer)?;
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

    if params.price_sol < 10_000_000{
        msg!("should larger than 100000000");
        return Err(ProgramError::InvalidArgument);
    }
    
    if params.name != params.name.trim().to_lowercase() {
        msg!("Domain names must be lower case and have no space");
        return Err(ProgramError::InvalidArgument);
    }
    if params.name.contains('.') {
        msg!("domain contains invalid puncation");
        return Err(ProgramError::InvalidArgument);
    }
    msg!("name: {}.{}", params.name, params.root_name);

    let rent = Rent::get()?;

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
            msg!("referrer is not unique");
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

    let hashed_name = get_hashed_name(&params.name);
    let (name_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key, 
        hashed_name.clone(), 
        None, 
        Some(accounts.root_domain.key)
    );
    check_account_key(accounts.domain_name_account, &name_account_key)?;
    msg!("name account ok");

    let hashed_reverse_lookup = get_hashed_name(&name_account_key.to_string());
    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_reverse_lookup.clone(),
        Some(&central_state::KEY),
        None,
    );
    check_account_key(accounts.reverse_lookup, &reverse_lookup_account_key)?;
    msg!("reverse account key ok");

    let name_state_account = accounts.domain_state_account;
    let (name_state_key, name_state_seeds) = get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(&params.name), 
        Some(&central_state::KEY), 
        Some(accounts.root_domain.key)
    );
    check_account_key(name_state_account, &name_state_key)?;
    msg!("name state ok");

    if !name_state_account.data_is_empty() {
        msg!("This domain name must be being auctioned.");
        return Err(ProgramError::InvalidArgument);
    }

    let name_state_lamports = rent.minimum_balance(NameStateRecordHeader::LEN);
    if name_state_lamports > params.price_sol {
        msg!("this can't be happend");
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

    let name_state_record = NameStateRecordHeader::new(
        accounts.fee_payer.key, 
        Clock::get()?.unix_timestamp, 
        params.price_sol,
        &params.root_name,
        &params.name,
    );
    name_state_record.pack_into_slice(& mut name_state_account.data.borrow_mut());
    msg!("write name state ok: {}.{}", params.name, params.root_name);

    if !accounts.domain_name_account.data_is_empty(){
        msg!("domain exsist");
        let domain_record = NameRecordHeader::unpack_from_slice(&accounts.domain_name_account.data.borrow())?;
        check_account_key(accounts.last_owner, &domain_record.owner)?;

        if domain_record.custom_price != params.price_sol {
            msg!("should be same as owner's custom price, custom: {}, you: {}", domain_record.custom_price, params.price_sol);
            return Err(ProgramError::InvalidArgument);
        }

        // directly transfer to vault, when the domain has settled, add profit to owner's profit
        invoke(
            &system_instruction::transfer(
                accounts.fee_payer.key, accounts.vault.key, math::sub(params.price_sol, name_state_lamports)?), 
                &[
                    accounts.fee_payer.clone(),
                    accounts.vault.clone(),
                    accounts.system_program.clone(),
                ]
        )?;
        msg!("transfer to vault: {:?} sol", math::sub(params.price_sol, name_state_lamports)?);
        
        let central_state_signer_seeds: &[&[u8]] = &[&crate::ID.to_bytes(), &[central_state::NONCE]];
        Cpi::change_preview(
            accounts.naming_service_program, 
            accounts.system_program, 
            accounts.domain_name_account, 
            accounts.root_domain, 
            accounts.central_state, 
            central_state_signer_seeds, 
            *accounts.fee_payer.key,
        )?;
    }else {

        invoke(
            &system_instruction::transfer(
                accounts.fee_payer.key, &vault_key, math::sub(params.price_sol, name_state_lamports)?), 
                &[
                    accounts.fee_payer.clone(),
                    accounts.vault.clone(),
                    accounts.system_program.clone(),
                ]
        )?;
        msg!("transfer all to vault: {:?} sol", math::sub(params.price_sol, name_state_lamports)?);

        let central_state_signer_seeds: &[&[u8]] = &[&crate::ID.to_bytes(), &[central_state::NONCE]];
        Cpi::create_name_account(
            accounts.naming_service_program, 
            accounts.system_program, 
            accounts.domain_name_account, 
            accounts.fee_payer, 
            accounts.central_state,
            accounts.root_domain, 
            accounts.central_state,
            accounts.fee_payer,
            hashed_name,
            rent.minimum_balance(NameRecordHeader::LEN as usize),
            central_state_signer_seeds,
            None,
        )?;

        if accounts.reverse_lookup.data_len() == 0 {
            Cpi::create_reverse_lookup_account(
                accounts.naming_service_program, 
                accounts.system_program, 
                accounts.reverse_lookup, 
                accounts.fee_payer, 
                params.name, 
                hashed_reverse_lookup, 
                accounts.central_state, 
                accounts.rent_sysvar, 
                central_state_signer_seeds, 
                None, 
                None
            )?;
        }
    }

    Ok(())
}
