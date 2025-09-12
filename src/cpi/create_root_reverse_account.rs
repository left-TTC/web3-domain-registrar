
use crate::state::ReverseLookup;

use super::Cpi;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed, program_pack::Pack, rent::Rent, sysvar::Sysvar
};
use borsh::BorshSerialize;
use web3_domain_name_service::{instruction::NameRegistryInstruction, state::NameRecordHeader};

impl Cpi {

    #[allow(clippy::too_many_arguments)]
    pub fn create_root_reverse_lookup_account<'a>(
        name_service_program: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        reverse_lookup_account: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        name: String,
        hashed_reverse_lookup: Vec<u8>,
        authority: &AccountInfo<'a>,
        rent_sysvar_account: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
        vault_seeds: &Vec<u8>,
    ) -> ProgramResult {
        let name_bytes = ReverseLookup { name }.try_to_vec().unwrap();
        let rent = Rent::from_account_info(rent_sysvar_account)?;
        let lamports = rent.minimum_balance(name_bytes.len() + NameRecordHeader::LEN);

        let create_name_instruction = web3_domain_name_service::instruction::create(
            *name_service_program.key,
            NameRegistryInstruction::Create {
                hashed_name: hashed_reverse_lookup,
                lamports,
                space: name_bytes.len() as u32,
                custom_value: None,
            },
            *reverse_lookup_account.key,
            *fee_payer.key,
            *authority.key,
            Some(*authority.key),
            None,
            None,
        )?;

        let accounts_create = vec![
            name_service_program.clone(),
            fee_payer.clone(),
            authority.clone(),
            reverse_lookup_account.clone(),
            system_program_account.clone(),
        ];

        let accounts_update = vec![
            name_service_program.clone(),
            reverse_lookup_account.clone(),
            authority.clone(),
        ];

        invoke_signed(
            &create_name_instruction, 
            &accounts_create, 
            &[
                &vault_seeds.chunks(32).collect::<Vec<&[u8]>>(),
                signer_seeds
            ])?;

        let write_name_instruction = web3_domain_name_service::instruction::update(
            *name_service_program.key,
            0,
            name_bytes,
            *reverse_lookup_account.key,
            *authority.key,
            None,
        )?;

        invoke_signed(&write_name_instruction, &accounts_update, &[signer_seeds])?;
        Ok(())
    }
}