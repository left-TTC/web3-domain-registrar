use web3_utils::declare_id_with_central_state;


pub mod entrypoint;
pub mod instruction_auto;
pub mod processor;
pub mod state;
pub mod utils;
pub mod cpi;


#[cfg(not(feature = "devnet"))]
declare_id_with_central_state!("jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR");

#[cfg(feature = "devnet")]
declare_id_with_central_state!("GMy1ikF3VFf4ffGq2tzNP3R2dc2mQij3fjVZVH7cFxZN");

#[cfg(feature = "devnet")]
pub mod constants {
        
    use solana_program::{pubkey, pubkey::Pubkey};

    pub const ADMIN_ANDY: Pubkey = pubkey!("DWNSuxCniY8m11DazRoN3VqvDZK8Sps2wgoQHWx3t4Sx");
    pub const ADMIN_FANMOCHENG: Pubkey = pubkey!("2NFji3XWVs2tb8btmGgkunjA9AFTr5x3DaTbsrZ7abGh");


    pub fn return_vault_key() -> (Pubkey, u8) {
        static VAULT_SEED: &[u8] = b"vault";

        let (vault_pda, bump) = Pubkey::find_program_address(&[VAULT_SEED], &crate::ID);

        (vault_pda, bump)
    }
}

#[cfg(not(feature = "devnet"))]
pub mod constants {
        
    use solana_program::{pubkey, pubkey::Pubkey};

    pub const ADMIN_ANDY: Pubkey = pubkey!("DWNSuxCniY8m11DazRoN3VqvDZK8Sps2wgoQHWx3t4Sx");
    pub const ADMIN_FANMOCHENG: Pubkey = pubkey!("2NFji3XWVs2tb8btmGgkunjA9AFTr5x3DaTbsrZ7abGh");


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