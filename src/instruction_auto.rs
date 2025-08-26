use crate::processor::{create, create_reverse, delete};
use web3_name_service_utils::InstructionsAccount;
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{instruction::Instruction, pubkey::Pubkey};
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, FromPrimitive)]


pub enum ProgramInstruction {
    Create,
    CreateReverse,
    Delete,
}


#[allow(missing_docs)]
pub fn create(
    program_id: Pubkey,
    accounts: create::Accounts<Pubkey>,
    params: create::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::Create as u8, params)
}
#[allow(missing_docs)]
pub fn create_reverse(
    program_id: Pubkey,
    accounts: create_reverse::Accounts<Pubkey>,
    params: create_reverse::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::CreateReverse as u8, params)
}

#[allow(missing_docs)]
pub fn delete(
    program_id: Pubkey,
    accounts: delete::Accounts<Pubkey>,
    params: delete::Params,
) -> Instruction {
    accounts.get_instruction(program_id, ProgramInstruction::Delete as u8, params)
}
