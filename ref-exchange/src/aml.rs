use near_sdk::{
    ext_contract, is_promise_success,
    json_types::{ValidAccountId, U128},
    serde::{Deserialize, Serialize},
    Promise,
};

use crate::*;

pub type CategoryRisk = (String, u8);

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum AmlOperation {
    Swap {
        actions: Vec<SwapAction>,
        referral_id: Option<ValidAccountId>,
    },
    AddLiquidity {
        pool_id: u64,
        amounts: Vec<U128>,
        min_amounts: Option<Vec<U128>>,
    },
    AddStableLiquidity {
        pool_id: u64,
        amounts: Vec<U128>,
        min_shares: U128,
    },
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn handle_refund(&mut self, sender_id: AccountId, attached_deposit: U128);

    fn callback_aml_operation(&mut self, operation: AmlOperation, sender_id: AccountId) -> U128;

    fn callback_ft_on_transfer(
        &mut self,
        token_in: AccountId,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> U128;
}

#[ext_contract(ext_aml)]
pub trait ExtAmlContract {
    fn get_address(&self, address: AccountId) -> CategoryRisk;
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn handle_refund(&mut self, sender_id: AccountId, attached_deposit: U128) {
        if !is_promise_success() {
            Promise::new(sender_id)
                .transfer(attached_deposit.0)
                .as_return();
        }
    }
}
