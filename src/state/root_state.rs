use solana_program::program_pack::Pack;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{Sealed},
    pubkey::Pubkey,
};

#[derive(Clone,Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct RootStateRecordHeader {
    pub root_flag: u8,
    pub initiator: Pubkey,
    pub amount: u64,
    pub name: [u8; 16],
}

impl Sealed for RootStateRecordHeader {}

impl RootStateRecordHeader {
    pub fn new(initiator: Pubkey, amount: u64, name: &str) -> Self {
        let mut buf = [0u8; 16];
        let raw = name.as_bytes();
        let len = raw.len().min(16);
        buf[..len].copy_from_slice(&raw[..len]);

        Self {
            root_flag: 1,
            initiator,
            amount,
            name: buf,
        }
    }
}

impl Pack for RootStateRecordHeader {
    const LEN: usize = 57;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut p = src;
        RootStateRecordHeader::deserialize(&mut p).map_err(|_| {
            msg!("Failed to deserialize name record");
            ProgramError::InvalidAccountData
        })
    }
}

