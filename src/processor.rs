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
pub mod register_root;
pub mod initialize_root;
pub mod finalize_name;
pub mod start_project;
pub mod extract_admin;
pub mod extract;

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
            ProgramInstruction::InitializeRoot => {
                msg!("Instruction: initiate root domain");
                let params = initialize_root::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                initialize_root::process_initialize_root(program_id, accounts, params)?;
            }
            ProgramInstruction::RegisterRoot => {
                msg!("Instruction: create root domain");
                let params = register_root::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                register_root::process_register_root(program_id, accounts, params)?;
            }
            ProgramInstruction::BeginNameRegistration => {
                msg!("Instruction: create name domain");
                let params = start_name::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                start_name::process_start_name(program_id, accounts, params)?;
            }
            ProgramInstruction::IncreaseBid => {
                msg!("Instruction: Participate in name auction");
                let params = increase_price::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                increase_price::process_increase_price(program_id, accounts, params)?;
            }
            ProgramInstruction::FinalizeName => {
                msg!("Instruction: settle and create an domain name");
                let params = finalize_name::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                finalize_name::process_finalize_name(program_id, accounts, params)?;
            }
            ProgramInstruction::Withdraw => {
                msg!("Instruction: user extract");
                let params = extract::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                extract::process_extract(program_id, accounts, params)?;
            }
            ProgramInstruction::InitializeProject => {
                msg!("Instruction: start Project");
                let params = start_project::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                start_project::process_start_project(program_id, accounts, params)?;
            }
            ProgramInstruction::WithdrawAdmin => {
                msg!("Instruction: admin extract");
                let params = extract_admin::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidArgument)?;
                extract_admin::process_extract_admin(program_id, accounts, params)?;
            }   
        }

        Ok(())
    }
}
