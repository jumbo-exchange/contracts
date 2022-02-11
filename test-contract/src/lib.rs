use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};

use near_sdk::collections::LookupMap;

const MAX_RISK: u8 = 10;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Category {
    None,
    Test,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Addresses,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct AddressInfo {
    category: Category,
    risk: u8,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ExtContract {
    owner_id: AccountId,
    pub addresses: LookupMap<AccountId, AddressInfo>,
}

#[near_bindgen]
impl ExtContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            addresses: LookupMap::new(StorageKey::Addresses),
        }
    }

    pub fn create_address(&mut self, address: AccountId, category: Category, risk: u8) {
        assert!(risk <= MAX_RISK, "Invalid risk");
        assert_eq!(
            self.addresses.contains_key(&address),
            false,
            "Address already exist"
        );
        let address_info = AddressInfo { category, risk };
        self.addresses.insert(&address, &address_info);
    }

    pub fn get_address(&self, address: AccountId) -> (Category, u8) {
        if let Some(address_info) = self.addresses.get(&address) {
            (address_info.category, address_info.risk)
        } else {
            (Category::None, 0)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Category;
    use near_sdk::{test_utils::VMContextBuilder, testing_env, MockedBlockchain};

    fn init_contract() -> ExtContract {
        testing_env!(VMContextBuilder::new().build());
        let contract = ExtContract::new("owner".to_string());
        contract
    }

    #[test]
    fn get_existing_address() {
        let mut contract = init_contract();
        let address: AccountId = "address_1".to_string();

        contract.create_address(address.clone(), Category::None, 0);

        let result = contract.get_address(address.clone());

        assert_eq!(result, (Category::None, 0));
    }

    #[test]
    #[should_panic(expected = "ERR_NOT_EXISTS")]
    fn get_non_existent_address() {
        let mut contract = init_contract();
        let address: AccountId = "address_1".to_string();

        contract.create_address("address".to_string(), Category::Test, 0);

        let result = contract.get_address(address.clone());

        assert_eq!(result, (Category::Test, 0), "ERR_NOT_EXISTS");
    }
}
