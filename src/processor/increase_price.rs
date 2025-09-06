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
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_program, sysvar,
    sysvar::Sysvar,
};
use web3_domain_name_service::state::{NameRecordHeader};
use spl_token::instruction::transfer;


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    pub space: u32,
    pub referrer_idx_opt: Option<u16>,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> {
    /// The naming service program ID
    pub naming_service_program: &'a T,
    /// The root domain account       
    pub root_domain: &'a T,
    /// The name account
    #[cons(writable)]
    pub name: &'a T,
    /// The reverse look up account   
    #[cons(writable)]
    pub reverse_lookup: &'a T,
    /// The system program account
    pub system_program: &'a T,
    /// The central state account
    pub central_state: &'a T,
    /// The buyer account     
    #[cons(writable, signer)]
    pub buyer: &'a T,
    /// The registered domain owner     
    pub domain_owner: &'a T,
    /// The solana fee payer account     
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    /// The buyer token account       
    #[cons(writable)]
    pub buyer_token_source: &'a T,
    /// The Pyth feed account
    pub pyth_feed_account: &'a T,
    /// The vault account     
    #[cons(writable)]
    pub vault: &'a T,
    /// The SPL token program
    pub spl_token_program: &'a T,
    /// The rent sysvar account
    pub rent_sysvar: &'a T,
    /// The state auction account
    pub state: &'a T,

    /// The *optional* referrer token account to receive a portion of fees.
    /// The token account owner has to be whitelisted.
    #[cons(writable)]
    pub referrer_account_opt: &'a T,
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
            name: next_account_info(accounts_iter)?,
            reverse_lookup: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            buyer: next_account_info(accounts_iter)?,
            domain_owner: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            buyer_token_source: next_account_info(accounts_iter)?,
            pyth_feed_account: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            // state: next_account_info(accounts_iter)?,
            referrer_account_opt: next_account_info(accounts_iter).ok(),
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_key(self.naming_service_program, &WEB3_NAME_SERVICE).unwrap();
        msg!("nameservice id ok");
        check_account_key(self.system_program, &system_program::ID).unwrap();
        msg!("system_program id ok");
        check_account_key(self.central_state, &central_state::KEY).unwrap();
        msg!("central_state id ok");
        check_account_key(self.spl_token_program, &spl_token::ID).unwrap();
        msg!("spl_token_program id ok");
        check_account_key(self.rent_sysvar, &sysvar::rent::ID).unwrap();
        msg!("rent_sysvar id ok");

        // Check ownership
        check_account_owner(self.name, &system_program::ID)
            .map_err(|_| crate::Error::AlreadyRegistered)?;
        msg!("rent_sysvar owner ok");

        check_account_owner(self.root_domain, &WEB3_NAME_SERVICE)?;
        msg!("root_domain owner ok");

        // Check signer
        check_signer(self.buyer).unwrap();
        msg!("buyer signature ok");
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
    check_vault_token_account_owner(accounts.vault).unwrap();
    
    if params.name != params.name.trim().to_lowercase() {
        msg!("Domain names must be lower case and have no space");
        return Err(ProgramError::InvalidArgument);
    }
    
    if params.name.contains('.') {
        return Err(ProgramError::InvalidArgument);
    }

    #[cfg(feature = "devnet")]
    msg!("root: {}", accounts.root_domain.key);
    msg!("name: {}", params.name);

    let (name_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key, 
        get_hashed_name(&params.name), 
        None, 
        Some(accounts.root_domain.key)
    );

    msg!("name account: {}", name_account_key);

    if &name_account_key != accounts.name.key {
        msg!("Provided wrong name account");
        return Err(ProgramError::InvalidArgument);
    }

    let hashed_reverse_lookup = get_hashed_name(&name_account_key.to_string());

    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_reverse_lookup.clone(),
        Some(&central_state::KEY),
        None,
    );

    if &reverse_lookup_account_key != accounts.reverse_lookup.key {
        msg!("Provided wrong reverse lookup account");
        return Err(ProgramError::InvalidArgument);
    }

    //auction state

    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state::NONCE]];

    let mut domain_token_price = get_domain_price(&params.name, &accounts)?;

    let token_acc = spl_token::state::Account::unpack(&accounts.buyer_token_source.data.borrow())?;
    if token_acc.mint == FWC_MINT {
        domain_token_price = domain_token_price.checked_mul(95).ok_or(Error::Overflow)? / 100;
    }

    //sns -- transfer coins to referrer

    //

    //transfer domain's price
    let transfer_ix = transfer(
        &spl_token::ID,
        accounts.buyer_token_source.key,
        accounts.vault.key,
        accounts.buyer.key,
        &[],
        domain_token_price,
    ).unwrap();

    invoke(
        &transfer_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.buyer_token_source.clone(),
            accounts.vault.clone(),
            accounts.buyer.clone(),
        ],
    ).unwrap();

    let rent = Rent::get()?;
    let hashed_name = get_hashed_name(&params.name);
    Cpi::create_name_account(
        accounts.naming_service_program,
        accounts.system_program,
        accounts.name,
        accounts.fee_payer,
        accounts.domain_owner,
        accounts.root_domain,
        accounts.central_state,
        hashed_name,
        rent.minimum_balance(NameRecordHeader::LEN + params.space as usize),
        params.space,
        central_state_signer_seeds,
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
            None,
        )?;
    }
    Ok(())
}
