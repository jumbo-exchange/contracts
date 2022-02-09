use near_sdk::{
    ext_contract,
    json_types::{ValidAccountId, U128},
    serde::{Deserialize, Serialize},
};

use crate::SwapAction;

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
    fn callback_is_aml_allowed(&self) -> bool;

    fn callback_aml_operation(
        &mut self,
        operation: AmlOperation,
        #[callback] is_aml_allowed: bool,
    ) -> U128;
}

#[ext_contract(ext_aml)]
pub trait ExtAmlContract {
    fn get_address(&self, address: AccountId) -> (String, u8);
}
