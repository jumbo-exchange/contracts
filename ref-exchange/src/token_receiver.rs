use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{serde_json, PromiseOrValue};

use crate::*;

pub const VIRTUAL_ACC: &str = "@";

pub const MIN_FT_EXECUTION_GAS: Gas = 50_000_000_000_000;

/// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
enum TokenReceiverMessage {
    /// Alternative to deposit + execute actions call.
    Execute {
        referral_id: Option<ValidAccountId>,
        /// List of sequential actions.
        actions: Vec<Action>,
    },
}

impl Contract {
    /// Executes set of actions on virtual account.
    /// Returns amounts to send to the sender directly.
    fn internal_direct_actions(
        &mut self,
        token_in: AccountId,
        amount_in: Balance,
        referral_id: Option<AccountId>,
        actions: &[Action],
    ) -> Vec<(AccountId, Balance)> {
        // let @ be the virtual account
        let mut account: Account = Account::new(&String::from(VIRTUAL_ACC));

        account.deposit(&token_in, amount_in);
        let _ = self.internal_execute_actions(
            &mut account,
            &referral_id,
            &actions,
            ActionResult::Amount(U128(amount_in)),
        );

        let mut result = vec![];
        for (token, amount) in account.tokens.to_vec() {
            if amount > 0 {
                result.push((token.clone(), amount));
            }
        }
        account.tokens.clear();

        result
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Callback on receiving tokens by this contract.
    /// `msg` format is either "" for deposit or `TokenReceiverMessage`.
    #[allow(unreachable_code)]
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.assert_contract_running();
        let token_in = env::predecessor_account_id();
        if msg.is_empty() {
            // Simple deposit.
            self.internal_deposit(sender_id.as_ref(), &token_in, amount.into());
            PromiseOrValue::Value(U128(0))
        } else {
            // instant swap
            let message =
                serde_json::from_str::<TokenReceiverMessage>(&msg).expect(ERR28_WRONG_MSG_FORMAT);
            match message {
                TokenReceiverMessage::Execute {
                    referral_id,
                    actions,
                } => {
                    let referral_id = referral_id.map(|x| x.to_string());

                    let prepaid_gas = env::prepaid_gas();
                    let required_gas = env::used_gas() + AML_CHECK_GAS + PROMISE_SCHEDULING_GAS;
                    assert!(
                        prepaid_gas >= required_gas + MIN_FT_EXECUTION_GAS,
                        "ERR_NOT_ENOUGH_GAS"
                    );
                    ext_aml::get_address(
                        sender_id.as_ref().clone(),
                        &self.aml_account_id,
                        0,
                        AML_CHECK_GAS,
                    )
                    .then(ext_self::callback_instant_swap(
                        token_in,
                        sender_id.into(),
                        amount,
                        referral_id,
                        actions,
                        &env::current_account_id(),
                        env::attached_deposit(),
                        prepaid_gas - required_gas,
                    ))
                    .into()
                }
            }
        }
    }
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn callback_instant_swap(
        &mut self,
        #[callback] category_risk: CategoryRisk,
        token_in: AccountId,
        sender_id: AccountId,
        amount: U128,
        referral_id: Option<AccountId>,
        actions: Vec<Action>,
    ) -> U128 {
        self.assert_risk(category_risk);

        let out_amounts = self.internal_direct_actions(token_in, amount.0, referral_id, &actions);
        for (token_out, amount_out) in out_amounts.into_iter() {
            self.internal_send_tokens(&sender_id, &token_out, amount_out);
        }
        // Even if send tokens fails, we don't return funds back to sender.
        U128(0)
    }
}
