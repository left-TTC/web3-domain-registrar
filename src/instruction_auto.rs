
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;



#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    InitiateRoot,
    CreateRoot,
    StartName,
    IncreasePrice,
    CreateName,
    StartProject,
    ConfirmRoot,
    Extract,
}

