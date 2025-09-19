
use super::Cpi;

use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program::invoke_signed,
};
use web3_domain_name_service::{instruction::NameRegistryInstruction};

impl Cpi {

    #[allow(clippy::too_many_arguments)]
    pub fn create_root_name_account<'a>(
        name_service_program: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        name_account: &AccountInfo<'a>,
        vault_pay: &AccountInfo<'a>,
        central_state_register_and_root_only_onwer: &AccountInfo<'a>,
        hashed_name: Vec<u8>,
        lamports: u64,
        signer_seeds: &Vec<u8>,
    ) -> ProgramResult {
        let create_name_instruction = web3_domain_name_service::instruction::create(
            *name_service_program.key,
            NameRegistryInstruction::Create {
                hashed_name,
                lamports,
                space: 0,
                custom_value: None,
            },
            *name_account.key,
            *vault_pay.key,
            *central_state_register_and_root_only_onwer.key,
            None,
            None,
            None,
        )?;

        invoke_signed(
            &create_name_instruction,
            &[
                name_service_program.clone(),
                vault_pay.clone(),
                name_account.clone(),
                central_state_register_and_root_only_onwer.clone(),
                system_program_account.clone(),
            ],
            &[&signer_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )
    }
}