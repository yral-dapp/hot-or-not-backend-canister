use std::{collections::HashMap, time::Duration};

use candid::{encode_one, Principal};
use ic_test_state_machine_client::WasmResult;
use pocket_ic::{PocketIc, PocketIcBuilder};
use shared_utils::{
    canister_specific::post_cache::types::arg::PostCacheInitArgs,
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
    test_constants::{
        get_global_super_admin_principal_id, get_mock_canister_id_sns,
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
        get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
    },
};

#[test]
fn reset_individual_canister_can_called_from_governance_canister_and_resetting_would_recharge_the_canister_if_needed_and_will_install_latest_wasm(
) {
    let state_machine = get_new_state_machine();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();

    //empty canister
    let alice_canister_id = state_machine.create_canister(Some(*user_index_canister_id));

    //cannot be called from global admin canister
    let res = state_machine.update_call(
        *user_index_canister_id,
        get_global_super_admin_principal_id(),
        "reset_user_individual_canisters",
        candid::encode_one(()).unwrap(),
    );

    assert!(res.is_err());

    state_machine
        .update_call(
            *user_index_canister_id,
            get_mock_canister_id_sns(),
            "reset_user_individual_canisters",
            candid::encode_one(vec![alice_canister_id]).unwrap(),
        )
        .unwrap();

    state_machine.advance_time(Duration::from_secs(30));
    state_machine.tick();
    state_machine.tick();
    state_machine.tick();
    state_machine.tick();
    state_machine.tick();
    state_machine.tick();
    state_machine.tick();

    let alice_canister_version = state_machine
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_version",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: String = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_version failed\n"),
            };
            response
        })
        .unwrap();

    assert!(alice_canister_version.eq("v1.0.0"));

    let availabe_canisters = state_machine
        .query_call(
            *user_index_canister_id,
            Principal::anonymous(),
            "get_list_of_available_canisters",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Vec<Principal> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_list_of_available_canisters failed\n"),
            };
            response
        })
        .unwrap();

    assert!(availabe_canisters.contains(&alice_canister_id));

    let alice_cycle_balance_after_reset = state_machine
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
        alice_cycle_balance_after_reset
    );

    assert!(alice_cycle_balance_after_reset >= 100_000_000_000);
}

const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";

const USER_INDEX_WASM_PATH: &str =
    "../../../../target/wasm32-unknown-unknown/release/user_index.wasm.gz";

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn user_index_canister_wasm() -> Vec<u8> {
    std::fs::read(USER_INDEX_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}

#[test]
fn reset_individual_canister_test() {
    let pic = PocketIc::new();
    let admin_principal_id = get_mock_user_charlie_principal_id();
    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();

    let individual_user_template = individual_template_canister_wasm();

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );
    known_prinicipal_values.insert(KnownPrincipalType::CanisterIdUserIndex, admin_principal_id);

    let post_cache_wasm_bytes = post_cache_canister_wasm();
    let post_cache_args = PostCacheInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        upgrade_version_number: Some(1),
        version: "1".to_string(),
    };
    let post_cache_args_bytes = encode_one(post_cache_args).unwrap();
    pic.install_canister(
        post_cache_canister_id,
        post_cache_wasm_bytes,
        post_cache_args_bytes,
        None,
    );
}
