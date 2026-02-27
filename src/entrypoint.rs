
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};
//Conditional compilation
#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;
use web3_domain_name_service::processor::Processor;
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);


pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint");
    Processor::process_instruction(program_id, accounts, instruction_data)?;
    Ok(())
}

