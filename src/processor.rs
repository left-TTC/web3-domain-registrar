use crate::instruction_auto::ProgramInstruction;
//Convert Rust data structures to and from binary data
use borsh::BorshDeserialize;
//math create:: fromprimitive  raw type->other types
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod start_name; 
pub mod increase_price;
pub mod create_root;
pub mod initiate_root;
pub mod settle_auction;

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
            ProgramInstruction::InitiateRoot => {
                msg!("Instruction: initiate root domain");
                let params = initiate_root::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                initiate_root::process_initiate_root(program_id, accounts, params)?;
            }
            ProgramInstruction::CreateRoot => {
                msg!("Instruction: create root domain");
                let params = create_root::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                create_root::process_create_root(program_id, accounts, params)?;
            }
            ProgramInstruction::StartName => {
                msg!("Instruction: create name domain");
                let params = start_name::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                start_name::process_start_name(program_id, accounts, params)?;
            }
            ProgramInstruction::IncreasePrice => {
                msg!("Instruction: Participate in name auction");
                let params = increase_price::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                increase_price::process_increase_price(program_id, accounts, params)?;
            }
            ProgramInstruction::CreateName => {
                msg!("Instruction: settle and create an domain name");
                let params = increase_price::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                increase_price::process_increase_price(program_id, accounts, params)?;
            }

            _ => {
                msg!("Instruction: Deprecated");
                return Err(ProgramError::InvalidInstructionData);
            }
        }

        Ok(())
    }
}
