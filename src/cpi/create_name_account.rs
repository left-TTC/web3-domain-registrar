
use super::Cpi;

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult, program::invoke_signed,
};
use web3_domain_name_service::instruction::NameRegistryInstruction;

impl Cpi {

    #[allow(clippy::too_many_arguments)]
    pub fn create_name_account<'a>(
        name_service_program: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        name_account: &AccountInfo<'a>,
        fee_payer_and_owner: &AccountInfo<'a>,
        root_name_account: &AccountInfo<'a>,
        authority: &AccountInfo<'a>,
        hashed_name: Vec<u8>,
        lamports: u64,
        signer_seeds: &[&[u8]],
        custom_value: Option<u64>
    ) -> ProgramResult {
        let create_name_instruction = web3_domain_name_service::instruction::create(
            *name_service_program.key,
            NameRegistryInstruction::Create {
                hashed_name,
                lamports,
                space: 0,
                custom_value,
            },
            *name_account.key,
            *fee_payer_and_owner.key,
            *fee_payer_and_owner.key,
            None,
            Some(*root_name_account.key),
            Some(*authority.key),
        )?;

        invoke_signed(
            &create_name_instruction,
            &[
                name_service_program.clone(),
                fee_payer_and_owner.clone(),
                name_account.clone(),
                system_program_account.clone(),
                root_name_account.clone(),
                authority.clone(),
            ],
            &[signer_seeds],
        )
    }
}