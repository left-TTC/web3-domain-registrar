use crate::instruction_auto::ProgramInstruction;
//Convert Rust data structures to and from binary data
use borsh::BorshDeserialize;
//math create:: fromprimitive  raw type->other types
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod create;
pub mod create_reverse;
pub mod delete;
pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        msg!("instruction: {:?}", instruction_data);

        let instruction = FromPrimitive::from_u8(instruction_data[0])
            .ok_or(ProgramError::InvalidInstructionData)?;
        let instruction_data = &instruction_data[1..];

        msg!("Instruction unpacked: means instruction data is ok");

        match instruction {
            ProgramInstruction::Create => {
                msg!("Instruction: Create web3 domain");
                let params = create::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create::process_create(program_id, accounts, params)?;
            }

            ProgramInstruction::CreateReverse => {
                msg!("Instruction: Create web3 domain Reverse account");
                let params = create_reverse::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_reverse::process_create_reverse(program_id, accounts, params)?;
            }

            ProgramInstruction::Delete => {
                msg!("Instruction: Delete web3 domain");
                let params = delete::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                delete::process_delete(program_id, accounts, params)?
            }
            _ => {
                msg!("Instruction: Deprecated");
                return Err(ProgramError::InvalidInstructionData);
            }
        }

        Ok(())
    }
}
