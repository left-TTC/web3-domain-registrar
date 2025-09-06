use solana_program::{
     hash::hashv, pubkey::Pubkey
};


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