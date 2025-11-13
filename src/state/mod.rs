
pub mod root_state;
pub mod name_state;
pub mod reverse_lookup;
pub mod referrer_record;
pub mod vault;

pub use root_state::*;
pub use name_state::*;
pub use reverse_lookup::*;
pub use referrer_record::*;

use solana_program::{
    account_info::AccountInfo,
};

pub fn write_data(account: &AccountInfo, input: &[u8], offset: usize) {
    let mut account_data = account.data.borrow_mut();
    let end = offset.saturating_add(input.len());
    account_data[offset..end].copy_from_slice(input);
}