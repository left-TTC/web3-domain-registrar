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
declare_id_with_central_state!("7WrKf6E5AHySxQjQ3fQBmhuw9VJMoPSXQo3qv8rdhpry");


#[cfg(feature = "devnet")]
pub mod constants {
        
    use solana_program::{pubkey, pubkey::Pubkey};

    //name service id: used to register domain
    pub const WEB3_NAME_SERVICE: Pubkey = pubkey!("BEUuE7dmW6yyTVhyecrHktKF1necQvfiZdFyBCBjCXQm");

    pub const WSOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

    pub const SYSTEM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

    pub const ADMIN: Pubkey = pubkey!("DWNSuxCniY8m11DazRoN3VqvDZK8Sps2wgoQHWx3t4Sx");

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