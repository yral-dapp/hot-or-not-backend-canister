use std::collections::BTreeMap;

use candid::Principal;
use ic_test_state_machine_client::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::post::Post,
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_bob_principal_id},
};

#[test]
fn error_when_owner_is_not_caller() {
    let state_machine = get_new_state_machine();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();
    let anonymous_principal_id = Principal::anonymous();
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

    let error_owner_is_not_caller = state_machine
        .update_call(
            alice_canister_id,
            anonymous_principal_id,
            "transfer_tokens_and_posts",
            candid::encode_one(alice_principal_id).unwrap(),
        )
        .map(|reply_payload| {
            let error_owner_is_not_caller: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 transfer_tokens_and_posts failed\n"),
            };
            error_owner_is_not_caller
        })
        .unwrap();

    assert_eq!(error_owner_is_not_caller, Err("Unauthorized caller".to_owned()));
}

#[test]
fn error_when_receiver_profiler_owner_is_not_receiver_caller() {
    let state_machine = get_new_state_machine();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();
    let anonymous_principal_id = Principal::anonymous();
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

    let posts: BTreeMap<u64, Post> = BTreeMap::new();

    let error_owner_is_not_caller = state_machine
        .update_call(
            alice_canister_id,
            anonymous_principal_id,
            "receive_data_from_hotornot",
            candid::encode_args((1000u64, alice_principal_id, posts)).unwrap(),
        )
        .map(|reply_payload| {
            let error_owner_is_not_caller: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 receive_data_from_hotornot failed\n"),
            };
            error_owner_is_not_caller
        })
        .unwrap();

    assert_eq!(error_owner_is_not_caller, Err("Unauthorized caller".to_owned()));
}

