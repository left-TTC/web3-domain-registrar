use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use web3_domain_name_service::{instruction::NameRegistryInstruction, state::NameRecordHeader};

use crate::state::ReverseLookup;

pub mod create_name_account;
pub mod create_root_name_account;
pub mod create_reverse_account;
pub mod create_root_reverse_account;

pub use create_name_account::*;
pub use create_root_name_account::*;
pub use create_reverse_account::*;
pub use create_root_reverse_account::*;

pub struct Cpi {}


