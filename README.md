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

# HAPI Protocol Integration

In first we add two properties on our context to store `aml_account_id` (HAPI Protocol contract id) and `accepted_risk_score` (a risk score that we set as safe). If risk score which we get from HAPI less or equal accepted_risk_score it means the address is safety.<br/>
[src/lib.rs](/ref-exchange/src/lib.rs#L80)
```rust
pub struct Contract {
    ...
    aml_account_id: AccountId,
    accepted_risk_score: u8,
}
```
Then we need to define a cross contract calls logic.<br/>
[src/aml.rs](/ref-exchange/src/aml.rs)
```rust
...
pub type CategoryRisk = (String, u8);

...

#[ext_contract(ext_self)]
pub trait ExtSelf {
    ...
    /// Callback with which we process the response from HAPI
    fn callback_aml_operation(&mut self, operation: AmlOperation, sender_id: AccountId) -> U128;

    ...
}

...

/// Trait for HAPI conract
#[ext_contract(ext_aml)]
pub trait ExtAmlContract {
    /// Method which return category type and risk for address
    fn get_address(&self, address: AccountId) -> CategoryRisk;
}
...
```
Finally, in each operation in which we need to check the address, we make a cross contract call to check it.<br/>
[src/lib.rs](/ref-exchange/src/lib.rs#L260)
 ```rust
 #[near_bindgen]
impl Contract {
    ...
    fn checked_aml_operation(&mut self, operation: AmlOperation) -> Promise {
        let prepaid_gas = env::prepaid_gas();
        // Calculated required gas for transaction
        // AML_CHECK_GAS (20TGas) - gas for call get_address in HAPI contract
        // PROMISE_SCHEDULING_GAS (25TGas) - gas for processing callbacks
        // REFUND_GAS (10TGas) - gas for second callback
        let required_gas = env::used_gas() + AML_CHECK_GAS + PROMISE_SCHEDULING_GAS + REFUND_GAS;
        // MIN_EXECUTION_GAS (20TGas) - min amount of gas to process callback_aml_operation
        assert!(
            prepaid_gas >= required_gas + MIN_EXECUTION_GAS,
            "ERR_NOT_ENOUGH_GAS"
        );
        // Call HAPI Protocol to check predecessor_account_id address
        ext_aml::get_address(
            env::predecessor_account_id(),
            &self.aml_account_id,
            0,
            AML_CHECK_GAS,
        )
        // In callback_aml_operation we first check the result
        .then(ext_self::callback_aml_operation(
            operation,
            env::predecessor_account_id(),
            &env::current_account_id(),
            env::attached_deposit(),
            prepaid_gas - required_gas,
        ))
        // Return attached deposit if promise is failure
        .then(ext_self::handle_refund(
            env::predecessor_account_id(),
            U128(env::attached_deposit()),
            &env::current_account_id(),
            0,
            REFUND_GAS,
        ))
    }
    ...

    pub fn callback_aml_operation(
        &mut self,
        #[callback] category_risk: CategoryRisk,
        operation: AmlOperation,
        sender_id: AccountId,
    ) {
        // Check risk score
        self.assert_risk(category_risk);
        ...
    }

    // If category not None and risk <= accepted_risk_score address is safety
    fn assert_risk(&self, category_risk: CategoryRisk) {
        let (category, risk) = category_risk;
        if category != "None" {
            assert!(risk <= self.accepted_risk_score, "ERR_AML_NOT_ALLOWED");
        };
    }
}
```
For example, let's check the `add_liquidity`.<br/>
Before, it's look like this.
```rust
    ...
    #[payable]
    pub fn add_liquidity(
        &mut self,
        pool_id: u64,
        amounts: Vec<U128>,
        min_amounts: Option<Vec<U128>>,
    ) {
        self.assert_contract_running();
        let sender_id = env::predecessor_account_id();
        // Adding liquidity without checking address
        self.internal_add_liquidity_unchecked(pool_id, amounts, sender_id, min_amounts);
    }
    ...
```
After, we add the `checked_aml_operation` function, to verify an address who call this method.
```rust
    ...
    #[payable]
    pub fn add_liquidity(
        &mut self,
        pool_id: u64,
        amounts: Vec<U128>,
        min_amounts: Option<Vec<U128>>,
    ) -> Promise {
        self.assert_contract_running();
        // Checking address before adding liquidity
        self.checked_aml_operation(AmlOperation::AddLiquidity {
            pool_id,
            amounts,
            min_amounts,
        })
    }
    ...
```
So, in `checked_aml_operation` we get info about address from HAPI
```rust
    fn checked_aml_operation(&mut self, operation: AmlOperation) -> Promise {
        ...
        ext_aml::get_address(
            env::predecessor_account_id(),
            &self.aml_account_id,
            0,
            AML_CHECK_GAS,
        )
        ...
    }
```
Then call `callback_aml_operation` to process the request.
```rust
    fn checked_aml_operation(&mut self, operation: AmlOperation) -> Promise {
        ...
        .then(ext_self::callback_aml_operation(
            operation,
            env::predecessor_account_id(),
            &env::current_account_id(),
            env::attached_deposit(),
            prepaid_gas - required_gas,
        ))
        ...
    }
```
First we check this info if all is ok, adding liquidity
```rust
    ...
    #[private]
    #[payable]
    pub fn callback_aml_operation(
        &mut self,
        #[callback] category_risk: CategoryRisk,
        operation: AmlOperation,
        sender_id: AccountId,
    ) {
        // Check info about address
        self.assert_risk(category_risk);
        // Adding liquidity
        self.aml_operation(operation, sender_id)
    }
    ...
```
# Testing 
```
cd ref-exchange
cargo test
```