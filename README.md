# Web3-domain-registrar


## Deploy
solana program deploy --program-id target/deploy/web3_domain_registrar-keypair.json target/sbf-so
lana-solana/release/web3_domain_registrar.so  --use-rpc

## Bulid
cargo build-sbf

## Account Structure
> About get_seeds_and_keys

|       Program_ID  |       Name_Class  |       Parent_Name |   Domain Type |
|       ----------  |       ----------  |       ----------- |   ----------- |
|WEB3_NAME_SERVICE  |       None        |       None        |   Root Domain |
|WEB3_NAME_SERVICE  |       Register_Central        |       None        |   Root Domain Reverse|
|WEB3_NAME_SERVICE  |       None        |       Root Domain Key        |   Name Domain |
|WEB3_NAME_SERVICE  |       Register_Central        |       None        |   Name Domain Reverse |
|WEB3_REGISTRAR  |       None        |       None        |   Root State Account |
|WEB3_REGISTRAR |       Register_Central        |       Root Domain        |   Name State Account |
|WEB3_REGISTRAR  |       Register_Central        |       Register_Central        |   Vault |
|WEB3_REGISTRAR  |       WEB3_REGISTRAR        |       WEB3_REGISTRAR        |   Referrer Record |

