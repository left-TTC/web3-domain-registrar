use web3_utils::declare_id_with_central_state;


pub mod entrypoint;
pub mod error;
pub mod instruction_auto;
pub mod processor;
pub mod state;
pub mod utils;
pub mod cpi;

pub use error::Error;

#[cfg(not(feature = "devnet"))]
declare_id_with_central_state!("jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR");

#[cfg(feature = "devnet")]
declare_id_with_central_state!("2xf73UX5CKCMwUznykZthaxnx2yq1QYjuNojatTeGfT7");


#[cfg(feature = "devnet")]
pub mod constants {
        
    use solana_program::{pubkey, pubkey::Pubkey};
    pub const SYSTEM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

    pub const ADMIN_ANDY: Pubkey = pubkey!("DWNSuxCniY8m11DazRoN3VqvDZK8Sps2wgoQHWx3t4Sx");
    pub const ADMIN_FANMOCHENG: Pubkey = pubkey!("2NFji3XWVs2tb8btmGgkunjA9AFTr5x3DaTbsrZ7abGh");

    // pub fn return_vault_key() -> (Pubkey, Vec<u8>){
    //     get_seeds_and_key(
    //         &crate::ID, 
    //         get_hashed_name("vault"), 
    //         Some(&central_state::KEY), 
    //         Some(&central_state::KEY)
    //     )
    // }

    pub fn return_vault_key() -> (Pubkey, u8) {
        static VAULT_SEED: &[u8] = b"vault";

        let (vault_pda, bump) = Pubkey::find_program_address(&[VAULT_SEED], &crate::ID);

        (vault_pda, bump)
    }
}

#[cfg(test)]
mod test {

    use solana_program::msg;
    use solana_system_interface::program::ID as id;
    #[test]
    fn test (){
        msg!("solana_system_interface::program::ID is {:?}", id);
    }
}