use solana_program::{hash::hashv};
use web3_domain_name_service::utils::HASH_PREFIX;

pub fn get_hashed_name(name: &str) -> Vec<u8> {
    hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec()
}


/// Check if root name conflicts with reserved domain names
pub fn is_reserved_root(root_name: &str) -> bool {
    let reserved_roots = [
        "com", "org", "net", "edu", "gov", "mil", "int",
        "io", "co", "uk", "us", "de", "fr", "cn", "jp",
        "au", "ca", "ru", "in", "br", "mx", "es", "it",
        "nl", "be", "ch", "se", "no", "dk", "fi", "pl",
        "kr", "tw", "hk", "sg", "my", "th", "vn", "ph",
        "id", "tr", "sa", "ae", "za", "eg", "ng", "ar",
        "cl", "co", "ve", "kr", "nz", "ie", "gr", "pt",
    ];
    
    let lower_root = root_name.to_lowercase();
    reserved_roots.contains(&lower_root.as_str())
}
