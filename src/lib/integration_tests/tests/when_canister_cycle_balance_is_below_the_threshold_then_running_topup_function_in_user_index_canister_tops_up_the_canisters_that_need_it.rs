use candid::Principal;
use ic_cdk::api::management_canister::main::CanisterStatusResponse;
use ic_state_machine_tests::{CanisterId, PrincipalId, StateMachine, WasmResult};
use shared_utils::common::types::known_principal::KnownPrincipalType;
use test_utils::setup::{
    env_v0::{
        get_canister_id_of_specific_type_from_principal_id_map,
        get_initialized_env_with_provisioned_known_canisters,
    },
    test_constants::{
        get_alice_principal_id, get_global_super_admin_principal_id,
        get_global_super_admin_principal_id_v1,
    },
};

#[test]
fn when_canister_cycle_balance_is_below_the_configured_or_freezing_threshold_then_running_topup_function_in_user_index_canister_tops_up_the_canisters_that_need_it(
) {
    // * Arrange
    let state_machine = StateMachine::new();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = get_canister_id_of_specific_type_from_principal_id_map(
        &known_principal_map,
        KnownPrincipalType::CanisterIdUserIndex,
    );
    let alice_principal_id = get_alice_principal_id();

    // * Act
    let alice_canister_id = state_machine.execute_ingress_as(
        alice_principal_id,
        user_index_canister_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let (alice_canister_id,): (Principal,) = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        alice_canister_id
    }).unwrap();

    let alice_starting_cycle_balance = state_machine
        .execute_ingress_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            user_index_canister_id,
            "get_canister_status_from_management_canister",
            candid::encode_one(alice_canister_id).unwrap(),
        )
        .map(|reply_payload| {
            let (response,): (CanisterStatusResponse,) = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
                _ => panic!("\n🛑 get_canister_status_from_management_canister failed\n"),
            };
            response.cycles
        })
        .unwrap();

    println!(
        "🧪 alice_starting_cycle_balance: {}",
        alice_starting_cycle_balance
    );

    state_machine
        .execute_ingress_as(
            get_global_super_admin_principal_id(),
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "return_cycles_to_user_index_canister",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();
    state_machine
        .execute_ingress_as(
            get_global_super_admin_principal_id(),
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "return_cycles_to_user_index_canister",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();

    let alice_reduced_cycle_balance = state_machine
        .execute_ingress_as(
            get_global_super_admin_principal_id(),
            user_index_canister_id,
            "get_canister_status_from_management_canister",
            candid::encode_one(alice_canister_id).unwrap(),
        )
        .map(|reply_payload| {
            let (response,): (CanisterStatusResponse,) = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
                _ => panic!("\n🛑 get_canister_status_from_management_canister failed\n"),
            };
            response.cycles
        })
        .unwrap();

    println!(
        "🧪 alice_reduced_cycle_balance: {}",
        alice_reduced_cycle_balance
    );

    state_machine
        .execute_ingress_as(
            get_global_super_admin_principal_id(),
            user_index_canister_id,
            "topup_canisters_that_need_it",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();

    let alice_topped_up_cycle_balance = state_machine
        .execute_ingress_as(
            get_global_super_admin_principal_id(),
            user_index_canister_id,
            "get_canister_status_from_management_canister",
            candid::encode_one(alice_canister_id).unwrap(),
        )
        .map(|reply_payload| {
            let (response,): (CanisterStatusResponse,) = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
                _ => panic!("\n🛑 get_canister_status_from_management_canister failed\n"),
            };
            response.cycles
        })
        .unwrap();

    println!(
        "🧪 alice_topped_up_cycle_balance: {}",
        alice_topped_up_cycle_balance
    );

    // * Assert
    assert!(alice_starting_cycle_balance > alice_reduced_cycle_balance);
    assert!(alice_reduced_cycle_balance < alice_topped_up_cycle_balance);

    // let state_machine = get_new_state_machine();
    // let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    // let user_index_canister_id = get_canister_id_of_specific_type_from_principal_id_map(
    //     &known_principal_map,
    //     KnownPrincipalType::CanisterIdUserIndex,
    // );
    // let alice_principal_id = get_alice_principal_id();
}
