


use solana_program::program_pack::Pack;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{Sealed},
    pubkey::Pubkey,
};


#[derive(Clone,Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct RefferrerRecordHeader {
    pub refferrer_account: Pubkey,
}

impl Sealed for RefferrerRecordHeader {}

impl RefferrerRecordHeader {
    pub fn new(refferrer: Pubkey) -> Self {
        Self { refferrer_account: refferrer }
    }
}

impl Pack for RefferrerRecordHeader {
    const LEN: usize = 32;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() != Self::LEN {
            msg!("refferrer record err");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self {
            refferrer_account: Pubkey::new_from_array(
                <[u8; 32]>::try_from(src).map_err(|_| ProgramError::InvalidAccountData)?
            )
        })
    }
}