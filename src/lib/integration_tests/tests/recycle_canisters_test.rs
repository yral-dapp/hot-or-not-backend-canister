use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use candid::{encode_args, encode_one, Principal};
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::PlaceBetArg,
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{BetDirection, BettingStatus},
            post::PostDetailsFromFrontend,
            profile::UserProfileDetailsForFrontend,
        },
        post_cache::types::arg::PostCacheInitArgs,
        user_index::types::{args::UserIndexInitArgs, RecycleStatus},
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
fn recycle_canisters_test() {
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
        user_index_wasm.clone(),
        user_index_args_bytes.clone(),
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
                _ => panic!("\n🛑 create_pool_of_individual_user_available_canisters failed\n"),
            };
            result
        })
        .unwrap();

    for _ in 0..15 {
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

    // Show alice rewards

    let alice_rewards = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice rewards: {:?}", alice_rewards);

    let alice_token_balance = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Alice token balance: {:?}", alice_token_balance);

    // Show bob rewards

    let bob_rewards = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob rewards: {:?}", bob_rewards);

    let bob_token_balance = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Bob token balance: {:?}", bob_token_balance);

    // Show dan rewards

    let dan_rewards = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Dan rewards: {:?}", dan_rewards);

    let dan_token_balance = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Dan token balance: {:?}", dan_token_balance);

    // upgrade user index canister

    for _ in 0..5 {
        pic.tick();
    }

    pic.upgrade_canister(
        user_index_canister_id,
        user_index_wasm.clone(),
        user_index_args_bytes,
        Some(admin_principal_id),
    )
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

    // print principal strs
    println!("Admin principal: {:?}", admin_principal_id.to_string());
    println!(
        "User Index principal: {:?}",
        user_index_canister_id.to_string()
    );
    println!(
        "Alice principal: {:?}",
        alice_individual_template_canister_id.to_string()
    );
    println!(
        "Bob principal: {:?}",
        bob_individual_template_canister_id.to_string()
    );
    println!(
        "Dan principal: {:?}",
        dan_individual_template_canister_id.to_string()
    );

    // call alice canister - get_last_canister_functionality_access_time
    let res1 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap()
        .unwrap();
    println!("Alice last access time: {:?}", res1);

    // call bob canister - get_last_canister_functionality_access_time
    let res2 = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap()
        .unwrap();
    println!("Bob last access time: {:?}", res2);

    // call dan canister - get_last_canister_functionality_access_time
    let res3 = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap()
        .unwrap();
    println!("Dan last access time: {:?}", res3);

    // Forward timer
    pic.advance_time(Duration::from_secs(7 * 24 * 60 * 60));
    for _ in 0..1000 {
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

    // call alice canister - get_last_canister_functionality_access_time
    let res11 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap()
        .unwrap();
    println!("Bob last access time: {:?}", res11);
    // calculate and diff in time between res1 and res11
    let diff = res11.duration_since(res1).unwrap();
    println!("Time diff: {:?}", diff);

    // call bob canister - get_last_canister_functionality_access_time
    let res22 = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap()
        .unwrap();
    println!("Bob last access time: {:?}", res22);
    // calculate and diff in time between res2 and res22
    let diff = res22.duration_since(res2).unwrap();
    println!("Time diff: {:?}", diff);

    // call dan canister - get_last_canister_functionality_access_time
    let res33 = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap()
        .unwrap();
    println!("Dan last access time: {:?}", res33);
    // calculate and diff in time between res3 and res33
    let diff = res33.duration_since(res3).unwrap();
    println!("Time diff: {:?}", diff);

    // call user_index get_recycle_status
    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
            "get_recycle_status",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let recycle_status: RecycleStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_recycle_status failed\n"),
            };
            recycle_status
        })
        .unwrap();
    println!("Recycle status: {:?}", res);

    // call user_index recycle_canisters
    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
            "recycle_canisters",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let recycle_status: () = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 recycle_canisters failed\n"),
            };
            recycle_status
        });

    for _ in 0..1000 {
        pic.tick();
    }

    // call user_index get_recycle_status
    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
            "get_recycle_status",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let recycle_status: RecycleStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_recycle_status failed\n"),
            };
            recycle_status
        })
        .unwrap();
    println!("Recycle status: {:?}", res);
}
