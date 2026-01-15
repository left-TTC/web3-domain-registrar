use solana_program::program_pack::Pack;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{Sealed},
};


#[derive(Clone,Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct VualltRecord {
    pub sill: u64,
    // after bid time 
    pub a_propotion: u8,
}

impl Sealed for VualltRecord {}

impl VualltRecord {
    pub fn new() -> Self {
        Self { 
            sill: 100_000_000,
            a_propotion: 52,
        }
    }
}

impl Pack for  VualltRecord {
    const LEN: usize = 9;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut p = src;
        VualltRecord::deserialize(&mut p).map_err(|_| {
            msg!("Failed to deserialize VualltRecord");
            ProgramError::InvalidAccountData
        })
    }
}
