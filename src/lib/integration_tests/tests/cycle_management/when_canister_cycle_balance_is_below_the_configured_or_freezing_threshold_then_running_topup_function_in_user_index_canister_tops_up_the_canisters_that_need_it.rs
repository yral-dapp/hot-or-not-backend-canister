use candid::Principal;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_test_state_machine_client::WasmResult;
use shared_utils::common::types::known_principal::KnownPrincipalType;
use test_utils::setup::{
    env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
    test_constants::{get_global_super_admin_principal_id_v1, get_mock_user_alice_principal_id},
};

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
            *user_index_canister_id,
            get_global_super_admin_principal_id_v1(),
            "get_canister_status_from_management_canister",
            candid::encode_one(alice_canister_id).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<CanisterStatusResponse, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_canister_status_from_management_canister failed\n"),
            };
            response.unwrap().cycles
        })
        .unwrap();

    println!(
        "🧪 alice_starting_cycle_balance: {}",
        alice_starting_cycle_balance
    );

    state_machine
        .update_call(
            alice_canister_id,
            get_global_super_admin_principal_id_v1(),
            "return_cycles_to_user_index_canister",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();
    state_machine
        .update_call(
            alice_canister_id,
            get_global_super_admin_principal_id_v1(),
            "return_cycles_to_user_index_canister",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();

    let alice_reduced_cycle_balance = state_machine
        .update_call(
            *user_index_canister_id,
            get_global_super_admin_principal_id_v1(),
            "get_canister_status_from_management_canister",
            candid::encode_one(alice_canister_id).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<CanisterStatusResponse, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_canister_status_from_management_canister failed\n"),
            };
            response.unwrap().cycles
        })
        .unwrap();

    assert!(alice_starting_cycle_balance > alice_reduced_cycle_balance);

    println!(
        "🧪 alice_reduced_cycle_balance: {}",
        alice_reduced_cycle_balance
    );

    state_machine
        .update_call(
            *user_index_canister_id,
            get_global_super_admin_principal_id_v1(),
            "topup_canisters_that_need_it",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();

    let alice_topped_up_cycle_balance = state_machine
        .update_call(
            *user_index_canister_id,
            get_global_super_admin_principal_id_v1(),
            "get_canister_status_from_management_canister",
            candid::encode_one(alice_canister_id).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<CanisterStatusResponse, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_canister_status_from_management_canister failed\n"),
            };
            response.unwrap().cycles
        })
        .unwrap();

    assert!(alice_reduced_cycle_balance < alice_topped_up_cycle_balance);

    println!(
        "🧪 alice_topped_up_cycle_balance: {}",
        alice_topped_up_cycle_balance
    );
}
