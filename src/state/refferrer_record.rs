use solana_program::program_pack::Pack;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::Sealed,
    pubkey::Pubkey,
};
use web3_domain_name_service::utils::get_seeds_and_key;

use crate::utils::get_hashed_name;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct RefferrerRecordHeader {
    pub refferrer_account: Pubkey, 
    pub profit: u64,               
    pub volume: u64,               
}

impl Sealed for RefferrerRecordHeader {}

impl RefferrerRecordHeader {
    pub fn new(refferrer: Pubkey) -> Self {
        Self {
            refferrer_account: refferrer,
            profit: 0,
            volume: 0,
        }
    }
}

impl Pack for RefferrerRecordHeader {
    const LEN: usize = 32 + 8 + 8;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        if dst.len() < Self::LEN {
            msg!("Invalid destination slice length for RefferrerRecordHeader");
            return;
        }

        let (pubkey_dst, rest) = dst.split_at_mut(32);
        pubkey_dst.copy_from_slice(self.refferrer_account.as_ref());

        let (profit_dst, volume_dst) = rest.split_at_mut(8);
        profit_dst.copy_from_slice(&self.profit.to_le_bytes());
        volume_dst.copy_from_slice(&self.volume.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < Self::LEN {
            msg!("Invalid data length for RefferrerRecordHeader");
            return Err(ProgramError::InvalidAccountData);
        }

        let (pubkey_src, rest) = src.split_at(32);
        let (profit_src, volume_src) = rest.split_at(8);

        let refferrer_account = Pubkey::new_from_array(
            <[u8; 32]>::try_from(pubkey_src)
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        let profit = u64::from_le_bytes(
            <[u8; 8]>::try_from(profit_src)
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        let volume = u64::from_le_bytes(
            <[u8; 8]>::try_from(volume_src)
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        Ok(Self {
            refferrer_account,
            profit,
            volume,
        })
    }
}

pub fn get_refferrer_record_key(usr: &Pubkey) -> (Pubkey, Vec<u8>) {
    get_seeds_and_key(
        &crate::ID,
        get_hashed_name(&usr.to_string()),
        Some(&crate::ID),
        Some(&crate::ID),
    )
}
