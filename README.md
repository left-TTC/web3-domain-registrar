# Web3-domain-registrar

## Overview
The main contract of Web3 Name Service
### Program Id
```
FebjdrGRLHocUXADP5QYPFfEkbYtRHMyobAL6wwkQe2d
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
|WEB3_REGISTRAR  |       WEB3_REGISTRAR        |       WEB3_REGISTRAR        |   referrer Record |


## Profit Sharing Ideas
### 1. Refferer
Every usr will set a refereer 
> We plan to distribute 91% of the profits to the referrers in the next three levels.

In this case, we assume that the buyer's name is A, and A's referrer is B, B's referrer is C, C's is D

#### Initial domain name creation
|Name|Expenditure|Income|Responsibility|
|---|---|---|---|
|A|ausume that X|None|pay for domain name and finally own the domain|
|B|None|x * 52%|get referral fees|
|C|None|x * 26%|get referral fees|
|D|None|x * 13%|get referral fees|

#### Secondary sale of domain name
95% of the sale amount belongs to the seller, and the remaining 5% will sitributed by the new owner's recommender according to the proportion


## Deploy
```bash
solana program deploy --program-id target/deploy/web3_domain_registrar-keypair.json target/sbpf-solana-solana/release/web3_domain_registrar.so  --use-rpc
```

```bash
cargo build-sbf
```

```bash
solana-keygen pubkey target/deploy/web3_domain_registrar-keypair.json
```

 