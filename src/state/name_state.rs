use solana_program::program_pack::Pack;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{Sealed},
    pubkey::Pubkey,
};


#[derive(Clone,Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct NameStateRecordHeader {
    pub highest_bidder: Pubkey,
    pub rent_payer: Pubkey,
    pub update_time: i64,
    pub highest_price: u64,
}

impl Sealed for NameStateRecordHeader {}

impl NameStateRecordHeader {
    pub fn new(
        highest_bidder: Pubkey, rent_payer: Pubkey, frist_time: i64, start_price: u64
    ) -> Self {
        Self { 
            highest_bidder: highest_bidder, 
            rent_payer: rent_payer,
            update_time: frist_time, 
            highest_price: start_price, 
        }
    }
}

impl Pack for  NameStateRecordHeader {
    const LEN: usize = 80;

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