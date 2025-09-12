use crate::processor::{start_name};
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;
use solana_program::{instruction::Instruction, pubkey::Pubkey};
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, FromPrimitive)]


pub enum ProgramInstruction {
    InitiateRoot,
    CreateRoot,
    StartName,
    IncreasePrice,
    CreateName,
}

