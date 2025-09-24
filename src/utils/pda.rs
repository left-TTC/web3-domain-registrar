use solana_program::{hash::hashv, msg, pubkey::Pubkey};
use web3_domain_name_service::utils::HASH_PREFIX;

pub fn get_hashed_name(name: &str) -> Vec<u8> {
    hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec()
}

pub fn get_seeds_and_keys(
    program_id: &Pubkey,
    hashed_name: Vec<u8>, 
    name_class_opt: Option<&Pubkey>,
    parent_name_address_opt: Option<&Pubkey>,
) -> (Pubkey, Vec<u8>) {
    let mut seeds_vec: Vec<u8> = hashed_name.clone();

    msg!("hashed name length: {:?}", hashed_name.len());

    let name_class = name_class_opt.cloned().unwrap_or_default();

    msg!("name_class length: {:?}", name_class.to_bytes().len());

    for b in name_class.to_bytes() {
        seeds_vec.push(b);
    }

    let parent_name_address = parent_name_address_opt.cloned().unwrap_or_default();

    msg!("parent_name_address length: {:?}", parent_name_address.to_bytes().len());

    for b in parent_name_address.to_bytes() {
        seeds_vec.push(b);
    }

    let (name_account_key, bump) =
        Pubkey::find_program_address(&seeds_vec.chunks(32).collect::<Vec<&[u8]>>(), program_id);
    seeds_vec.push(bump);

    msg!("seeds : {:?}", seeds_vec);

    (name_account_key, seeds_vec)
}