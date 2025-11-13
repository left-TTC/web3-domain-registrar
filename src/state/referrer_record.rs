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
/// The record for a referrer account — stores earnings and performance
pub struct ReferrerRecordHeader {
    /// The wallet address of the referrer
    pub referrer_account: Pubkey,
    /// The total profit this referrer earned (lamports or tokens)
    pub profit: u64,
    /// The total performance (sales volume -- only the subordinates' share is calculated.)
    pub performance: u64,
    /// When this record was created (Unix timestamp, seconds)
    pub create_time: i64,
}

impl Sealed for ReferrerRecordHeader {}

impl ReferrerRecordHeader {
    pub fn new(referrer: Pubkey, create_time: i64) -> Self {
        Self {
            referrer_account: referrer,
            profit: 0,
            performance: 0,
            create_time,
        }
    }
}

impl Pack for ReferrerRecordHeader {
    /// 32 (Pubkey) + 8 (profit) + 8 (performance) + 8 (create_time)
    const LEN: usize = 32 + 8 + 8 + 8;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        if dst.len() < Self::LEN {
            msg!("Invalid destination slice length for ReferrerRecordHeader");
            return;
        }

        let (pubkey_dst, rest) = dst.split_at_mut(32);
        pubkey_dst.copy_from_slice(self.referrer_account.as_ref());

        let (profit_dst, rest) = rest.split_at_mut(8);
        profit_dst.copy_from_slice(&self.profit.to_le_bytes());

        let (performance_dst, create_time_dst) = rest.split_at_mut(8);
        performance_dst.copy_from_slice(&self.performance.to_le_bytes());

        create_time_dst.copy_from_slice(&self.create_time.to_le_bytes());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < Self::LEN {
            msg!("Invalid data length for ReferrerRecordHeader");
            return Err(ProgramError::InvalidAccountData);
        }

        let (pubkey_src, rest) = src.split_at(32);
        let (profit_src, rest) = rest.split_at(8);
        let (performance_src, create_time_src) = rest.split_at(8);

        let referrer_account = Pubkey::new_from_array(
            <[u8; 32]>::try_from(pubkey_src)
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        let profit = u64::from_le_bytes(
            <[u8; 8]>::try_from(profit_src)
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        let performance = u64::from_le_bytes(
            <[u8; 8]>::try_from(performance_src)
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        let create_time = i64::from_le_bytes(
            <[u8; 8]>::try_from(create_time_src)
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        Ok(Self {
            referrer_account,
            profit,
            performance,
            create_time,
        })
    }
}

/// Derive PDA for a referrer record
pub fn get_referrer_record_key(usr: &Pubkey) -> (Pubkey, Vec<u8>) {
    get_seeds_and_key(
        &crate::ID,
        get_hashed_name(&usr.to_string()),
        Some(&crate::ID),
        Some(&crate::ID),
    )
}
