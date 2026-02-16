use solana_program::program_pack::Pack;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::Sealed,
    pubkey::Pubkey,
};


#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ValuableDomain {
    /// Domain account pubkey
    pub domain: Pubkey,
    /// Domain value
    pub value: u64,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct VaultRecord {
    /// Dividend proportion
    pub a_proportion: u8,
    /// Total number of created domains
    pub domain_count: u32,
    /// Current number of valid high-value domains
    pub top_len: u8,
    /// Top 6 most valuable domains
    pub top_domains: [ValuableDomain; 6],
}

impl Sealed for VaultRecord {}

impl VaultRecord {
    pub fn new() -> Self {
        Self {
            a_proportion: 52,
            domain_count: 0,
            top_len: 0,
            top_domains: core::array::from_fn(|_| ValuableDomain {
                domain: Pubkey::default(),
                value: 0,
            }),
        }
    }

    /// Update top domains if new domain value is higher than the lowest
    pub fn update_top_domain(&mut self, domain: Pubkey, value: u64) {
        // If not at capacity, add the domain
        if self.top_len < 6 {
            self.top_domains[self.top_len as usize] = ValuableDomain { domain, value };
            self.top_len += 1;
            return;
        }

        // Find the minimum value in the top domains
        let min_index = self.top_domains
            .iter()
            .enumerate()
            .min_by_key(|(_, d)| d.value)
            .map(|(i, _)| i)
            .unwrap_or(0);

        // Replace if new value is higher than the minimum
        if value > self.top_domains[min_index].value {
            self.top_domains[min_index] = ValuableDomain { domain, value };
        }
    }
}

impl Pack for VaultRecord {
    // u8 (1) + u32 (4) + u8 (1) + 6 * (32 + 8) = 1 + 4 + 1 + 240 = 246
    const LEN: usize = 1 + 4 + 1 + (6 * (32 + 8));

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut p = src;
        VaultRecord::deserialize(&mut p).map_err(|_| {
            msg!("Failed to deserialize VaultRecord");
            ProgramError::InvalidAccountData
        })
    }
}
