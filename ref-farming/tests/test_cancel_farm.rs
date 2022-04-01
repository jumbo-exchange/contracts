use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, to_yocto, view};
use ref_farming::{FarmInfo, HRSimpleFarmTerms};

use crate::common::actions::*;
use crate::common::init::deploy_farming;
use crate::common::utils::*;

mod common;

#[test]
fn test_cancel_farm() {
    let root = init_simulator(None);

    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let farmer1 = root.create_user("farmer1".to_string(), to_yocto("100"));

    let (pool, token1, _) = prepair_pool_and_liquidity(&root, &owner, farming_id(), vec![]);

    let farming = deploy_farming(&root, farming_id(), owner.account_id());

    let seed_id = format!("{}@0", pool.account_id());
    let farm_id = format!("{}#0", seed_id.clone());

    call!(
        owner,
        farming.create_simple_farm(
            HRSimpleFarmTerms {
                seed_id: seed_id.clone(),
                reward_token: token1.valid_account_id(),
                start_at: 0,
                reward_per_session: to_yocto("1").into(),
                session_interval: 60,
            },
            None
        ),
        deposit = to_yocto("1")
    )
    .assert_success();

    assert_err!(
        call!(farmer1, farming.cancel_farm(farm_id.clone())),
        "ERR_NOT_ALLOWED"
    );

    assert_err!(
        call!(owner, farming.cancel_farm("random".to_string())),
        "E41: farm not exist"
    );

    //add reward
    mint_token(&token1, &root, to_yocto("10"));
    call!(
        root,
        token1.storage_deposit(Some(to_va(farming_id())), None),
        deposit = to_yocto("1")
    )
    .assert_success();
    call!(
        root,
        token1.ft_transfer_call(
            to_va(farming_id()),
            U128(to_yocto("10")),
            None,
            farm_id.clone()
        ),
        deposit = 1
    )
    .assert_success();

    //The rewards have been handed out, but farm not expire
    root.borrow_runtime_mut().cur_block.block_timestamp += to_nano(60 * 11);
    assert_err!(
        call!(owner, farming.cancel_farm(farm_id.clone())),
        "This farm can NOT be cancelled"
    );

    root.borrow_runtime_mut().cur_block.block_timestamp += to_nano(3600 * 24 * 30);
    assert_err!(
        call!(owner, farming.cancel_farm(farm_id.clone())),
        "This farm can NOT be cancelled"
    );

    call!(
        owner,
        farming.create_simple_farm(
            HRSimpleFarmTerms {
                seed_id: seed_id.clone(),
                reward_token: token1.valid_account_id(),
                start_at: 0,
                reward_per_session: to_yocto("1").into(),
                session_interval: 60,
            },
            None
        ),
        deposit = to_yocto("1")
    )
    .assert_success();

    let farms = view!(farming.list_farms_by_seed(seed_id.clone())).unwrap_json::<Vec<FarmInfo>>();
    let outdated_farms = view!(farming.list_outdated_farms(0, 100)).unwrap_json::<Vec<FarmInfo>>();

    assert_eq!(farms.len(), 2);
    assert_eq!(outdated_farms.len(), 0);

    call!(
        owner,
        farming.cancel_farm(format!("{}@0#1", pool.account_id()))
    )
    .assert_success();

    let farms = view!(farming.list_farms_by_seed(seed_id)).unwrap_json::<Vec<FarmInfo>>();
    let outdated_farms = view!(farming.list_outdated_farms(0, 100)).unwrap_json::<Vec<FarmInfo>>();

    assert_eq!(farms.len(), 1);
    assert_eq!(outdated_farms.len(), 0);
}
