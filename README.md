# Web3-domain-registrar


## Deploy
```bash
solana program deploy --program-id target/deploy/web3_domain_registrar-keypair.json target/sbf-solana-solana/release/web3_domain_name_
service.so  --use-rpc
```

## Bulid
```bash
cargo build-sbf
```
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
|WEB3_REGISTRAR  |       WEB3_REGISTRAR        |       None        |   Name State Reverse |

## Profit Sharing Ideas

### 1. Refferer
Every usr will set a refereer 
> We plan to distribute 90% of the profits to the referrers in the next three levels（only create）.

### 2. rent exemption
When create a domain auction state, the sponsor needs to pay the rent exemption, but when the domain is twice auctioned, rent exemption has beed paid by the sponor, it's unfair
> I plan to distribute 2% of the profits to the sponor for his distribution in every auction, and we will get the 1%  

