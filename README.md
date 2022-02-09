# Jumbo Contracts

## CLI installation

You can install cli via this [tutorial](https://docs.near.org/docs/tools/near-cli#installation).


## Getting started

For creating the new account for deploying contract to testnet use next command
```
near create-account subAccount.yourAccount.testnet --masterAccount yourAccount.testnet --initialBalance 10
```


Create constants
```
export CONTRACT_ID=subAccount.yourAccount.testnet
export OWNER_ID=yourAccount.testnet
```


First of all - you will need to compile the wasm file of contracts
```
cd ref-exchange
./build_local.sh
```


And then deploy it like that
```
near deploy $CONTRACT_ID --wasmFile=res/ref_exchange_local.wasm
```


Then initialize contract with command
```
near call $CONTRACT_ID new '{"owner_id": "'$OWNER_ID'", "exchange_fee":"5", "referral_fee": 10}' --accountId $CONTRACT_ID
```

## Testing 
```
cd ref-exchange
cargo test
```