use web3_name_service_utils::declare_id_with_central_state;


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
declare_id_with_central_state!("Hyk2fr7w4Tyf19jKFUCW35aDBkCkcBbadEU12RDdbDKx");


#[cfg(feature = "devnet")]
pub mod constants {
        
    use solana_program::{pubkey, pubkey::Pubkey};

    //name service id: used to register domain
    pub const WEB3_NAME_SERVICE: Pubkey = pubkey!("DqynrcXcYhfJbUYQZZFq6A2Tx64cJQGwyufWJxLpnKsK");

    pub const WSOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

}


#[cfg(test)]
mod test {

    #[test]
    fn test (){
        let test_array: [u8; 11] = [13, 2, 0, 0, 0, 97, 97, 0, 4, 0, 0,];
    }
}