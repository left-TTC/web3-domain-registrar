
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
        fee_payer: &AccountInfo<'a>,
        central_state_register: &AccountInfo<'a>,
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
            *fee_payer.key,
            *central_state_register.key,
            None,
            None,
            None,
        )?;

        invoke_signed(
            &create_name_instruction,
            &[
                name_service_program.clone(),
                fee_payer.clone(),
                name_account.clone(),
                central_state_register.clone(),
                system_program_account.clone(),
            ],
            &[&signer_seeds.chunks(32).collect::<Vec<&[u8]>>()],
        )
    }
}