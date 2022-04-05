use near_sdk::json_types::ValidAccountId;
use near_sdk::AccountId;
use std::convert::TryFrom;

pub(crate) fn dai() -> AccountId {
    "dai".to_string()
}

pub(crate) fn eth() -> AccountId {
    "eth".to_string()
}

pub(crate) fn swap() -> AccountId {
    "swap".to_string()
}

pub(crate) fn farming_id() -> AccountId {
    "farming".to_string()
}

pub(crate) fn aml_id() -> AccountId {
    "aml".to_string()
}

pub(crate) fn to_va(a: AccountId) -> ValidAccountId {
    ValidAccountId::try_from(a).unwrap()
}

pub(crate) fn to_nano(timestamp: u32) -> u64 {
    u64::from(timestamp) * 10u64.pow(9)
}

#[macro_export]
macro_rules! assert_err {
    (print $exec_func: expr) => {
        println!(
            "{:?}",
            $exec_func.promise_errors()[0].as_ref().unwrap().status()
        );
    };
    ($exec_func: expr, $err_info: expr) => {
        assert!(format!(
            "{:?}",
            $exec_func.promise_errors()[0].as_ref().unwrap().status()
        )
        .contains($err_info));
    };
}
