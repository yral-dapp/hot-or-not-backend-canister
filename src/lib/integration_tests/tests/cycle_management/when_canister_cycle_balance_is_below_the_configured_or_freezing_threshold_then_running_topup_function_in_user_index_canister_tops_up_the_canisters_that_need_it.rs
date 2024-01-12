use std::time::Duration;

use candid::Principal;
use ic_test_state_machine_client::WasmResult;
use shared_utils::{
    canister_specific::user_index::types::args::UserIndexInitArgs,
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
    test_constants::{
        get_canister_wasm, get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
    },
};

#[ignore]
#[test]
fn when_canister_cycle_balance_is_below_the_configured_or_freezing_threshold_then_running_topup_function_in_user_index_canister_tops_up_the_canisters_that_need_it(
) {
    let state_machine = get_new_state_machine();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();
    let alice_principal_id = get_mock_user_alice_principal_id();

    let alice_canister_id = state_machine.update_call(
        *user_index_canister_id,
        alice_principal_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let alice_canister_id: Principal = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        alice_canister_id
    }).unwrap();

    let alice_starting_cycle_balance = state_machine
        .update_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_user_caniser_cycle_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: u128 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_caniser_cycle_balance failed\n"),
            };
            response
        })
        .unwrap();

    println!(
        "🧪 alice_starting_cycle_balance: {}",
        alice_starting_cycle_balance
    );

    state_machine
        .update_call(
            alice_canister_id,
            *user_index_canister_id,
            "return_cycles_to_user_index_canister",
            candid::encode_one(Some(600_000_000_000_u128)).unwrap(),
        )
        .unwrap();

    let alice_reduced_cycle_balance = state_machine
        .update_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_user_caniser_cycle_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: u128 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_caniser_cycle_balance failed\n"),
            };
            response
        })
        .unwrap();

    println!(
        "🧪 alice_reduced_cycle_balance: {}",
        alice_reduced_cycle_balance
    );

    assert!(alice_starting_cycle_balance > alice_reduced_cycle_balance);

    state_machine
        .upgrade_canister(
            *user_index_canister_id,
            get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
            candid::encode_one(UserIndexInitArgs {
                ..Default::default()
            })
            .unwrap(),
            Some(get_global_super_admin_principal_id()),
        )
        .unwrap();

    state_machine.advance_time(Duration::from_secs(30));
    state_machine.tick();

    let alice_cycle_balance_after_user_index_upgrade = state_machine
        .update_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_user_caniser_cycle_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: u128 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_caniser_cycle_balance failed\n"),
            };
            response
        })
        .unwrap();

    println!(
        "🧪 alice_cycle_balance_after_user_index_upgrade: {}",
        alice_cycle_balance_after_user_index_upgrade
    );

    assert!(alice_cycle_balance_after_user_index_upgrade >= 600_000_000_000);
}
