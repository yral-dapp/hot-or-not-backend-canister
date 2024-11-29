use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime},
};

use candid::{encode_args, encode_one, CandidType, Principal};
use ic_cdk::api::management_canister::main::{CanisterId, CanisterSettings};
use ic_ledger_types::{AccountIdentifier, BlockIndex, Tokens, DEFAULT_SUBACCOUNT};
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::PlaceBetArg,
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{BetDirection, BettingStatus},
            ml_data::MLFeedCacheItem,
            post::PostDetailsFromFrontend,
            profile::UserProfileDetailsForFrontend,
        },
        platform_orchestrator::types::args::PlatformOrchestratorInitArgs,
        post_cache::types::arg::PostCacheInitArgs,
        user_index::types::{args::UserIndexInitArgs, RecycleStatus},
    },
    common::types::{known_principal::KnownPrincipalType, wasm::WasmType},
    constant::{NNS_CYCLE_MINTING_CANISTER, NNS_LEDGER_CANISTER_ID},
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
        get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
        v1::CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    },
};

const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";

const USER_INDEX_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/user_index.wasm.gz";
const PF_ORCH_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/platform_orchestrator.wasm.gz";

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn user_index_canister_wasm() -> Vec<u8> {
    std::fs::read(USER_INDEX_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}
fn pf_orch_canister_wasm() -> Vec<u8> {
    std::fs::read(PF_ORCH_WASM_PATH).unwrap()
}

// TODO: remove this when removing the update_last_access_time API from PF Orch

#[test]
fn reset_ml_feed_cache_test() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let charlie_global_admin = get_mock_user_charlie_principal_id();
    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "add_principal_as_global_admin",
            candid::encode_one(charlie_global_admin).unwrap(),
        )
        .unwrap();

    let user_index_canister_id: Principal = pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[1]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for i in 0..50 {
        pocket_ic.tick();
    }

    // User Index available details - call get_subnet_available_capacity

    let res = pocket_ic
        .query_call(
            user_index_canister_id,
            super_admin,
            "get_subnet_available_capacity",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let subnet_capacity: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_subnet_available_capacity failed\n"),
            };
            subnet_capacity
        })
        .unwrap();
    println!("Avail capacity: {:?}", res);

    // call get_subnet_backup_capacity

    let res = pocket_ic
        .query_call(
            user_index_canister_id,
            super_admin,
            "get_subnet_backup_capacity",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let subnet_capacity: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_subnet_backup_capacity failed\n"),
            };
            subnet_capacity
        })
        .unwrap();
    println!("Backup capacity: {:?}", res);

    // create user canisters

    let alice_individual_template_canister_id = pocket_ic
        .update_call(
            user_index_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            result
        })
        .unwrap()
        .unwrap();
    println!("res1: {:?}", alice_individual_template_canister_id);

    let bob_individual_template_canister_id = pocket_ic
        .update_call(
            user_index_canister_id,
            bob_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            result
        })
        .unwrap()
        .unwrap();
    println!("res2: {:?}", bob_individual_template_canister_id);

    let dan_individual_template_canister_id = pocket_ic
        .update_call(
            user_index_canister_id,
            dan_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            result
        })
        .unwrap()
        .unwrap();
    println!("res3: {:?}", dan_individual_template_canister_id);

    // add items into ml cache

    let items = vec![
        MLFeedCacheItem {
            post_id: 1 as u64,
            canister_id: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
            video_id: "1".to_string(),
            creator_principal_id: None,
        },
        MLFeedCacheItem {
            post_id: 2 as u64,
            canister_id: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
            video_id: "2".to_string(),
            creator_principal_id: None,
        },
        MLFeedCacheItem {
            post_id: 3 as u64,
            canister_id: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
            video_id: "3".to_string(),
            creator_principal_id: None,
        },
    ];

    let res = pocket_ic
        .update_call(
            alice_individual_template_canister_id,
            user_index_canister_id,
            "update_ml_feed_cache",
            candid::encode_one(items.clone()).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<String, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    let res1 = pocket_ic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_ml_feed_cache_paginated",
            encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let result: Vec<MLFeedCacheItem> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();

    assert_eq!(res1.len(), 3);

    let res = pocket_ic
        .update_call(
            bob_individual_template_canister_id,
            user_index_canister_id,
            "update_ml_feed_cache",
            candid::encode_one(items).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<String, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    let res2 = pocket_ic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_ml_feed_cache_paginated",
            encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let result: Vec<MLFeedCacheItem> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();

    assert_eq!(res2.len(), 3);

    // reset cache

    let res = pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "reset_canisters_ml_feed_cache",
            candid::encode_one(application_subnets[1]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<String, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for i in 0..50 {
        pocket_ic.tick();
    }

    let res1 = pocket_ic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_ml_feed_cache_paginated",
            encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let result: Vec<MLFeedCacheItem> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();

    assert_eq!(res1.len(), 0);

    let res2 = pocket_ic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_ml_feed_cache_paginated",
            encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let result: Vec<MLFeedCacheItem> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();

    assert_eq!(res2.len(), 0);
}
