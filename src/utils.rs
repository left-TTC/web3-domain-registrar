use crate::{
    central_state,
    constants::{ VAULT_OWNER, WEB3_NAME_SERVICE },
    processor::create,
};
use web3_name_service_utils::{
    checks::{check_account_key, check_account_owner},
    fp_math::fp32_div,
    tokens::SupportedToken,
};

use solana_program::{
    account_info::AccountInfo, clock::Clock, hash::hashv, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey, sysvar::Sysvar,
};

use spl_token::state::Account;
use unicode_segmentation::UnicodeSegmentation;

////////////////////////////////////////////////////////////

pub const HASH_PREFIX: &str = "WEB3 Name Service";

pub fn get_hashed_name(name: &str) -> Vec<u8> {
    hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec()
}

// can be use from name service
pub fn get_seeds_and_key(
    program_id: &Pubkey,
    hashed_name: Vec<u8>, // Hashing is done off-chain
    name_class_opt: Option<&Pubkey>,
    parent_name_address_opt: Option<&Pubkey>,
) -> (Pubkey, Vec<u8>) {
    let mut seeds_vec: Vec<u8> = hashed_name;

    let name_class = name_class_opt.cloned().unwrap_or_default();
    for b in name_class.to_bytes() {
        seeds_vec.push(b);
    }

    let parent_name_address = parent_name_address_opt.cloned().unwrap_or_default();
    for b in parent_name_address.to_bytes() {
        seeds_vec.push(b);
    }

    let (name_account_key, bump) =
        Pubkey::find_program_address(&seeds_vec.chunks(32).collect::<Vec<&[u8]>>(), program_id);
    seeds_vec.push(bump);

    (name_account_key, seeds_vec)
}

////////////////////////////////////////////////////////////

pub fn get_usd_price(len: usize) -> u64 {
    let multiplier = match len {
        1 => 750,
        2 => 700,
        3 => 640,
        4 => 160,
        _ => 20,
    };
    #[cfg(not(feature = "devnet"))]
    return multiplier * 1_000_000;
    #[cfg(feature = "devnet")]
    return multiplier * 1_000;
}

pub fn get_grapheme_len(name: &str) -> usize {
    name.graphemes(true).count()
}

//calculate web3 name account key
pub fn get_name_key(name: &str, parent: Option<&Pubkey>) -> Result<Pubkey, ProgramError> {
    let hashed_name = get_hashed_name(name);
    let (name_account_key, _) = get_seeds_and_key(
        &WEB3_NAME_SERVICE,
        hashed_name,
        None,
        parent,
    );
    Ok(name_account_key)
}

//calculate web3 domain name reverse key --> get the domain name by name account key
pub fn get_reverse_key(
    domain_key: &Pubkey,
    parent_key: Option<&Pubkey>,
) -> Result<Pubkey, ProgramError> {
    let hashed_reverse_lookup = get_hashed_name(&domain_key.to_string());
    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        &WEB3_NAME_SERVICE,
        hashed_reverse_lookup,
        Some(&central_state::KEY),
        parent_key,
    );
    Ok(reverse_lookup_account_key)
}

////////////////////////////////////////////////////////////

pub struct PythAccounts<'a, 'b> {
    pub pyth_mapping_acc_or_feed: &'a AccountInfo<'b>,
    pub buyer_token_mint: Pubkey,
}

impl<'a, 'b: 'a> From<&'_ create::Accounts<'a, AccountInfo<'b>>> for PythAccounts<'a, 'b> {
    fn from(value: &create::Accounts<'a, AccountInfo<'b>>) -> Self {
        let buyer_token_mint =
            spl_token::state::Account::unpack_from_slice(&value.buyer_token_source.data.borrow())
                .unwrap()
                .mint;
        Self {
            pyth_mapping_acc_or_feed: value.pyth_feed_account,
            buyer_token_mint,
        }
    }
}

//Get the required number of tokens
pub fn get_domain_price <'a, 'b: 'a>(
    domain_name: &str,
    accounts: &create::Accounts<'a, AccountInfo<'b>>,
) -> Result<u64, ProgramError> {
    //get price by domain's length
    let usd_price = get_usd_price(get_grapheme_len(domain_name));
    //get buyer's designated mint token account
    let buyer_token_mint =
        spl_token::state::Account::unpack_from_slice(&accounts.buyer_token_source.data.borrow())
            .unwrap()
            .mint;

    let token_price =
        get_domain_price_check(accounts.pyth_feed_account, &buyer_token_mint)?;
    let domain_price = fp32_div(usd_price, token_price).unwrap();

    Ok(domain_price)
}

pub fn get_domain_price_check(
    pyth_feed: &AccountInfo<'_>,
    mint: &Pubkey,
) -> Result<u64, ProgramError> {
    let token = SupportedToken::from_mint(mint)?;
    check_account_key(pyth_feed, &token.price_feed_account_key())?;
    let token_price = bonfida_utils::pyth::get_oracle_price_fp32_v2(
        mint,
        pyth_feed,
        token.decimals(),
        6,
        &Clock::get().unwrap(),
        6000,
    )?;
    Ok(token_price)
}

pub fn check_vault_token_account_owner(account: &AccountInfo) -> Result<Account, ProgramError> {
    check_account_owner(account, &spl_token::ID)?;
    let token_account = Account::unpack_from_slice(&account.data.borrow())?;

    if token_account.owner != VAULT_OWNER {
        return Err(ProgramError::IllegalOwner);
    }

    Ok(token_account)
}

#[test]
pub fn test_length() {

}
