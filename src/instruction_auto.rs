use crate::processor::{start_name};
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

