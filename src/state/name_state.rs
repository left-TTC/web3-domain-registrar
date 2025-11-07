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
    pub highest_bidder: Pubkey,
    pub update_time: i64,
    pub highest_price: u64,
    // after bid time 
    pub settled: bool,
}

impl Sealed for NameStateRecordHeader {}

impl NameStateRecordHeader {
    pub fn new(
        highest_bidder: &Pubkey, update_time: i64, highest_price: u64, 
    ) -> Self {
        Self { 
            highest_bidder: *highest_bidder, 
            update_time: update_time, 
            highest_price: highest_price, 
            // new() means start an Auction
            settled: false,
        }
    }
}

impl Pack for  NameStateRecordHeader {
    const LEN: usize = 49;

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