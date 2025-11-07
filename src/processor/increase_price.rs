//! Create a domain name and buy the ownership of a domain name

use web3_utils::{
    accounts::InstructionsAccount, 
    borsh_size::BorshSize, 
    check::{check_account_key, check_account_owner, check_signer}, 
    BorshSize, 
    InstructionsAccount
};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::ProgramResult, 
    msg, 
    program::{invoke, invoke_signed}, 
    program_error::ProgramError, 
    program_pack::Pack, 
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar, 
};

use solana_system_interface::instruction as system_instruction;

use crate::{constants::{SYSTEM_ID, return_vault_key}, state::{NameStateRecordHeader, RefferrerRecordHeader, get_name_state_key, get_refferrer_record_key}, utils::{TIME, check_state_time, get_now_time, share}};


#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    pub my_price_sol: u64,
    pub refferrer_key: Pubkey,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> {
    /// The root domain account       
    pub root_domain: &'a T,
    /// The domain auction state account
    #[cons(writable)]
    pub domain_state_account: &'a T,
    /// The system program account
    pub system_program: &'a T,
    /// The buyer account         
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    // it's not necessary to confirm the refferrer
    /// last bidder
    #[cons(writable)]
    pub last_bidder: &'a T,
    /// the vault
    #[cons(writable)]
    pub vault: &'a T,
    /// usr's refferrer record
    /// we must check it, otherwise, some users may not have a referer in the end.
    #[cons(writable)]
    pub refferrer_record_account: &'a T,
    /// refferrer's refferrer record account
    pub superior_refferrer_record: Option<&'a T>,
    /// if need create the refferrer record, we need the rent sysvar
    pub rent: Option<&'a T>,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            root_domain: next_account_info(accounts_iter)?,
            domain_state_account: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            last_bidder: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            refferrer_record_account: next_account_info(accounts_iter)?,
            superior_refferrer_record: next_account_info(accounts_iter).ok(),
            rent: next_account_info(accounts_iter).ok(),
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {

        check_account_key(self.system_program, &SYSTEM_ID).unwrap();
        msg!("system_program id ok");

        check_account_owner(self.root_domain, &web3_domain_name_service::ID)?;
        check_account_owner(self.domain_state_account, &crate::ID)
            .map_err(|_| crate::Error::AlreadyRegistered)?;

        check_signer(self.fee_payer).unwrap();
        msg!("fee_payer signature ok");

        Ok(())
    }
}


pub fn process_increase_price<'a, 'b: 'a>(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {

    let accounts = Accounts::parse(accounts)?;
    accounts.check()?;

    let name_state_account = accounts.domain_state_account;
    let (name_state_key, _) = 
        get_name_state_key(&params.name, accounts.root_domain.key);
    
    check_account_key(name_state_account, &name_state_key)?;
    msg!("name state key right");

    let name_state_data = 
        NameStateRecordHeader::unpack_from_slice(&name_state_account.data.borrow())?;
    
    if check_state_time(name_state_data.update_time)? != TIME::AUCTION {
        msg!("This auction has settled");
        return Err(ProgramError::InvalidArgument);
    }
    if params.my_price_sol < share(name_state_data.highest_price, 101)? {
        msg!("At least 1% markup");
        return Err(ProgramError::InvalidArgument);
    }

    let vault = accounts.vault;
    let (vault_key, _) = return_vault_key();
    check_account_key(vault, &vault_key)?;
    msg!("vault ok");

    // the referreer record account
    let refferrer_record_account = accounts.refferrer_record_account;
    let (refferrer_record, refferrer_seeds) = get_refferrer_record_key(&accounts.fee_payer.key);
    check_account_key(refferrer_record_account, &refferrer_record)?;
    msg!("payer's refferrer record account ok");

    if accounts.refferrer_record_account.data_len() == 0 {
        msg!("new user, should init the refferrer account");

        if let Some(rent_sysvar) = accounts.rent {
            let rent = Rent::from_account_info(rent_sysvar)?;
            let refferrer_record_lamports = rent.minimum_balance(RefferrerRecordHeader::LEN);

            if params.refferrer_key != vault_key {
                msg!("payer use other's invitation");
                if let Some(superior_refferrer) = accounts.superior_refferrer_record {
                    let (superior_refferrer_key, _) = get_refferrer_record_key(&params.refferrer_key);
                    check_account_key(superior_refferrer, &superior_refferrer_key)?;

                    msg!("refferr's refferrer record account ok");
                    
                    let _state =  
                        RefferrerRecordHeader::unpack_from_slice(&superior_refferrer.data.borrow())?;
                    msg!("refeerrer's refferrer is valid");
                } else {
                    msg!("you should provide your refferrer's refferrer record while your refferrer isn't vault"); 
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

            let record = RefferrerRecordHeader::new(
                params.refferrer_key
            );

            let mut data = accounts.refferrer_record_account.try_borrow_mut_data()?;
            record.pack_into_slice(&mut data);

            msg!("write in refferrer record account");
        }else {
            msg!("should give a rent");
            return Err(ProgramError::InvalidArgument);
        }

    }else {
        let buyer_refferrer_record =
            RefferrerRecordHeader::unpack_from_slice(&accounts.refferrer_record_account.data.borrow())?;

        if buyer_refferrer_record.refferrer_account != params.refferrer_key {
            msg!("the refferrer you provied is fault");
            return Err(ProgramError::InvalidArgument);
        }
        msg!("refferrer ok");
    }


    // transfer back the deposit
    invoke(&system_instruction::transfer(
        accounts.fee_payer.key, accounts.last_bidder.key, name_state_data.highest_price), 
        &[
            accounts.fee_payer.clone(),
            accounts.last_bidder.clone(),
            accounts.system_program.clone(),
        ]
    )?;
    //transfer the increased part to vault
    invoke(&system_instruction::transfer(
        accounts.fee_payer.key, &vault_key, params.my_price_sol - name_state_data.highest_price), 
        &[
            accounts.fee_payer.clone(),
            accounts.vault.clone(),
            accounts.system_program.clone(),
        ]
    )?;

    let new_record = NameStateRecordHeader::new(
        accounts.fee_payer.key, 
        get_now_time()?, 
        params.my_price_sol,
    );
    NameStateRecordHeader::pack(new_record, &mut name_state_account.data.borrow_mut())?;
    msg!("update the name record ok");

    Ok(())
}
