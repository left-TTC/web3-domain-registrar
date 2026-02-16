use solana_program::program_pack::Pack;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{Sealed},
    pubkey::Pubkey,
};
use web3_domain_name_service::utils::get_seeds_and_key;

use crate::{central_state, utils::get_hashed_name};


#[derive(Clone,Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct NameStateRecordHeader {
    /// The public key of the highest bidder
    pub highest_bidder: Pubkey,
    /// The timestamp of the last update
    pub update_time: i64,
    /// The highest bid price
    pub highest_price: u64,
    /// Fixed root domain name
    pub root: [u8; 16],
    /// Subdomain name
    pub name: [u8; 32],
}

impl Sealed for NameStateRecordHeader {}

impl NameStateRecordHeader {
    pub fn new(
        highest_bidder: &Pubkey, update_time: i64, highest_price: u64, root: &str, name: &str
    ) -> Self {
        let mut root_buf = [0u8; 16];
        let root_bytes = root.as_bytes();
        let root_len = root_bytes.len().min(16);
        root_buf[..root_len].copy_from_slice(&root_bytes[..root_len]);

        let mut name_buf = [0u8; 32];
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len().min(32);
        name_buf[..name_len].copy_from_slice(&name_bytes[..name_len]);

        Self { 
            highest_bidder: *highest_bidder, 
            update_time: update_time, 
            highest_price: highest_price, 
            root: root_buf,
            name: name_buf,
        }
    }
}

impl Pack for  NameStateRecordHeader {
    const LEN: usize = 96;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut p = src;
        NameStateRecordHeader::deserialize(&mut p).map_err(|_| {
            msg!("Failed to deserialize name state record");
            ProgramError::InvalidAccountData
        })
    }
}

pub fn get_name_state_key(
    domain_sub_name: &String,
    root_domain_key: &Pubkey,
) -> (Pubkey, Vec<u8>) {
    get_seeds_and_key(
        &crate::ID, 
        get_hashed_name(domain_sub_name), 
        Some(&central_state::KEY), 
        Some(root_domain_key)
    )
}