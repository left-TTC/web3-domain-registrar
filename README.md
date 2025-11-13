# Web3-domain-registrar


## Deploy
```bash
solana program deploy --program-id target/deploy/web3_domain_registrar-keypair.json target/sbf-solana-solana/release/web3_domain_registrar.so  --use-rpc
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
|WEB3_REGISTRAR  |       WEB3_REGISTRAR        |       WEB3_REGISTRAR        |   referrer Record |
|WEB3_REGISTRAR  |       WEB3_REGISTRAR        |       None        |   Name State Reverse |

> About account structure

|Account Type|Param 1|Param 2|Param 3|Param 4|Size|
|---|---|---|---|---|---|
|Name(root)|Parent Name(None)|owner(centarl registrar)|class(None)|custom price(meaningless)|104|
|Name(reverse)|Parent Name(None)|owner(centarl registrar)|class(centarl registrar)|custom price(meaningless)|104 + name.len|
|Name(common)|Parent Name(Root)|owner(usr)|class(None)|custom price(resale)|104|
|Name(reverse)|Parent Name(None)|owner(centarl registrar)|class(centarl registrar)|custom price(meaningless)|104 + name.len|

## Profit Sharing Ideas

### 1. Refferer
Every usr will set a refereer 
> We plan to distribute 90% of the profits to the referrers in the next three levels (only create).
#### Specific profit sharing ratio
> In this case, we assume that the buyer's name is A, and A's referrer is B, B's referrer is C, C's is D

##### Initial domain name creation
|Name|Expenditure|Income|Responsibility|
|---|---|---|---|
|A|ausume that X(now is $1.99)|None|pay for domain name|
|B|None|x * 40%|get referral fees|
|C|None|x * 30%|get referral fees|
|D|None|x * 20%|get referral fees|

### 2. rent exemption
When create a domain auction state, the sponsor needs to pay the rent exemption, but when the domain is twice auctioned, rent exemption has beed paid by the sponor, it's unfair
> We will distribute 3% of the profits to the sponor for his distribution in every auction
- As long as the transaction is initiated, there will be profit regardless of whether it is successful or not
- No maximium limitation

 