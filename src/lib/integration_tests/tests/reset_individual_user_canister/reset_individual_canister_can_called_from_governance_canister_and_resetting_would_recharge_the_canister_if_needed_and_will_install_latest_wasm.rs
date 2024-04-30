use std::collections::HashMap;

use candid::{encode_args, encode_one, Principal};
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::PlaceBetArg,
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{BetDirection, BettingStatus},
            post::PostDetailsFromFrontend,
            session::SessionType,
        },
        post_cache::types::arg::PostCacheInitArgs,
        user_index::types::args::UserIndexInitArgs,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
};

const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";

const USER_INDEX_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/user_index.wasm.gz";

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
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdSnsGovernance,
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

    let user_index_canister_id = pic.create_canister_with_settings(Some(admin_principal_id), None);
    pic.add_cycles(user_index_canister_id, 2_000_000_000_000_000);
    let user_index_wasm = user_index_canister_wasm();
    let user_index_args = UserIndexInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        access_control_map: None,
        version: "1".to_string(),
    };
    let user_index_args_bytes = encode_one(user_index_args).unwrap();
    pic.install_canister(
        user_index_canister_id,
        user_index_wasm,
        user_index_args_bytes,
        Some(admin_principal_id),
    );

    // Individual template canisters
    let individual_template_wasm_bytes = individual_template_canister_wasm();

    let res = pic
        .update_call(
            user_index_canister_id,
            admin_principal_id,
            "create_pool_of_individual_user_available_canisters",
            encode_args(("1".to_string(), individual_template_wasm_bytes)).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            result
        })
        .unwrap();

    for i in 0..20 {
        pic.tick();
    }

    // User Index available details - call get_subnet_available_capacity

    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
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

    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
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

    let alice_individual_template_canister_id = pic
            .update_call(
                user_index_canister_id,
                alice_principal_id,
                "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
                encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let result: Principal = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
                };
                result
            })
            .unwrap();
    println!("res1: {:?}", alice_individual_template_canister_id);

    let bob_individual_template_canister_id = pic
            .update_call(
                user_index_canister_id,
                bob_principal_id,
                "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
                encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let result: Principal = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
                };
                result
            })
            .unwrap();
    println!("res2: {:?}", bob_individual_template_canister_id);

    let dan_individual_template_canister_id = pic
            .update_call(
                user_index_canister_id,
                dan_principal_id,
                "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
                encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let result: Principal = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
                };
                result
            })
            .unwrap();
    println!("res3: {:?}", dan_individual_template_canister_id);

    pic.add_cycles(alice_individual_template_canister_id, 2_000_000_000_000);
    pic.add_cycles(bob_individual_template_canister_id, 2_000_000_000_000);
    pic.add_cycles(dan_individual_template_canister_id, 2_000_000_000_000);

    // User 1 creates posts
    let alice_post_1 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res1 = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(alice_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    let alice_post_2 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch 2".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#12345".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res2 = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(alice_post_2).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // Top up Bob's account
    let reward = pic.update_call(
        bob_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // Top up Dan's account
    let reward = pic.update_call(
        dan_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // Bob places bet on Alice post 1
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(bob_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Bob places bet on Alice post 2
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(bob_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 1
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pic
        .update_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(dan_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 2
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pic
        .update_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(dan_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // call update_session_type on dan_individual_template_canister_id by admin_principal_id
    let res = pic
        .update_call(
            dan_individual_template_canister_id,
            user_index_canister_id,
            "update_session_type",
            encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .map(|reply_payload| {
            let res: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_session_type failed\n"),
            };
            res.unwrap()
        })
        .unwrap();

    // User Index available details - call get_subnet_available_capacity

    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
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

    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
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

    // reset canisters

    let res = pic
        .update_call(
            user_index_canister_id,
            admin_principal_id,
            "reset_user_individual_canisters",
            encode_args((vec![
                alice_individual_template_canister_id,
                bob_individual_template_canister_id,
                dan_individual_template_canister_id,
            ],))
            .unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            result
        })
        .unwrap();

    for i in 0..50 {
        pic.tick();
    }

    // User Index available details - call get_subnet_available_capacity

    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
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
    assert_eq!(res, 19);

    // call get_subnet_backup_capacity

    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
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
    assert_eq!(res, 10);

    // call list of availble canisters on user_index
    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
            "get_list_of_available_canisters",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canisters: Vec<Principal> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_individual_canisters failed\n"),
            };
            canisters
        })
        .unwrap();
    assert_eq!(res.len(), 19);
    assert!(res.contains(&alice_individual_template_canister_id));

    // call get_version on alice_individual_template_canister_id
    let res = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_version",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let version: String = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_version failed\n"),
            };
            version
        })
        .unwrap();
    assert_eq!(res, "1");
}
