

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::{invoke_signed}, pubkey::Pubkey
};

use crate::cpi::Cpi;


impl Cpi {
    pub fn transfer_name_account<'a>(
        name_service_program: &AccountInfo<'a>,
        central_state: &AccountInfo<'a>,
        name_account: &AccountInfo<'a>,
        root_domain_account: &AccountInfo<'a>,
        new_owner_key: &Pubkey,
        signer_seeds: &[&[u8]],
        custom_value: Option<u64>
    ) -> ProgramResult {
        let transfer_name_instruction = web3_domain_name_service::instruction::transfer(
            *name_service_program.key,
            *new_owner_key,
            *name_account.key,
            *central_state.key,
            *root_domain_account.key,
            custom_value
        )?;

        invoke_signed(
            &transfer_name_instruction,
            &[
                name_service_program.clone(),
                central_state.clone(),
                name_account.clone(),
            ],
            &[signer_seeds],
        )
    }
}