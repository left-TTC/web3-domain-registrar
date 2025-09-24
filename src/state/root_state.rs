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
    pub initiator: Pubkey,
    pub amount: u64,
    pub name: [u8; 32],
}

impl Sealed for RootStateRecordHeader {}

impl RootStateRecordHeader {
    pub fn new(initiator: Pubkey, amount: u64, name: &str) -> Self {
        let mut buf = [0u8; 32];
        let raw = name.as_bytes();
        let len = raw.len().min(32);
        buf[..len].copy_from_slice(&raw[..len]);

        Self {
            initiator,
            amount,
            name: buf,
        }
    }

    // pub fn get_name(&self) -> String {
    //     let end = self.name.iter().position(|&c| c == 0).unwrap_or(self.name.len());
    //     String::from_utf8_lossy(&self.name[..end]).to_string()
    // }
}

impl Pack for RootStateRecordHeader {
    const LEN: usize = 72;

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

