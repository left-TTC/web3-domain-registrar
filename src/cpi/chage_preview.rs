
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult, program::invoke_signed, pubkey::Pubkey,
};
use crate::cpi::Cpi;




impl Cpi {
    

    #[allow(clippy::too_many_arguments)]
    pub fn change_preview<'a>(
        name_service_program: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        name_account: &AccountInfo<'a>,
        root_name_account: &AccountInfo<'a>,
        authority: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
        new_preview: Pubkey,
    ) -> ProgramResult {
        let change_preview_instruction = web3_domain_name_service::instruction::change_preview(
            *name_service_program.key,
            *name_account.key,
            *root_name_account.key,
            *authority.key,
            new_preview,
        )?;

        invoke_signed(
            &change_preview_instruction,
            &[
                name_service_program.clone(),
                name_account.clone(),
                root_name_account.clone(),
                authority.clone(),
                system_program_account.clone(),
            ],
            &[signer_seeds],
        )
    }
}