use std::{collections::HashMap, time::Duration};

use candid::{encode_one, Principal};
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::{FolloweeArg, IndividualUserTemplateInitArgs, PlaceBetArg},
<<<<<<< HEAD
            error::{
                BetOnCurrentlyViewingPostError, FollowAnotherUserProfileError,
                GetPostsOfUserProfileError,
            },
            follow::{FollowEntryDetail, FollowEntryId},
            hot_or_not::{BetDirection, BettingStatus, PlacedBetDetail},
            post::{PostDetailsForFrontend, PostDetailsFromFrontend},
            profile::UserProfileDetailsForFrontend,
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::types::{known_principal::KnownPrincipalType, utility_token::token_event::TokenEvent},
    constant::RECLAIM_CANISTER_PRINCIPAL_ID,
    types::canister_specific::individual_user_template::error_types::GetUserUtilityTokenTransactionHistoryError,
=======
            error::{BetOnCurrentlyViewingPostError, FollowAnotherUserProfileError},
            hot_or_not::{BetDirection, BettingStatus},
            post::PostDetailsFromFrontend,
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::types::known_principal::KnownPrincipalType,
    constant::RECLAIM_CANISTER_PRINCIPAL_ID,
>>>>>>> 18666a8... fix for key str
};
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
};

const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";


#[cfg(feature = "bet_details_heap_to_stable_mem_upgrade")]
#[test]
fn download_snapshot_test() {
    let pic = PocketIc::new();

    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();
    let admin_principal_id = get_mock_user_charlie_principal_id();

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

    // Individual template canisters
    let individual_template_wasm_bytes = individual_template_canister_wasm();

    // Init individual template canister - alice

    let alice_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(alice_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(alice_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        alice_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init individual template canister - bob

    let bob_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(bob_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(bob_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        bob_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init individual template canister - dan

    let dan_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(dan_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(dan_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        dan_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Create posts
    // Alice creates a post

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
        bet_amount: 50,
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

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60 * 2));
    pic.tick();

    // Bob places bet on Alice post 2
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
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

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60 * 2));
    pic.tick();

    // Dan places bet on Alice post 1
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 200,
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

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60 * 2));
    pic.tick();

    // Dan places bet on Alice post 2
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 50,
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

    // Follow each other

    // Alice follows Bob
    let follow_arg = FolloweeArg {
        followee_principal_id: bob_principal_id,
        followee_canister_id: bob_individual_template_canister_id,
    };
    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "update_profiles_i_follow_toggle_list_with_specified_profile",
            encode_one(follow_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            bet_status.unwrap()
        })
        .unwrap();
    assert_eq!(res, true);

    // Alice follows Dan
    let follow_arg = FolloweeArg {
        followee_principal_id: dan_principal_id,
        followee_canister_id: dan_individual_template_canister_id,
    };
    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "update_profiles_i_follow_toggle_list_with_specified_profile",
            encode_one(follow_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            bet_status.unwrap()
        })
        .unwrap();
    assert_eq!(res, true);

    // Bob follows Alice
    let follow_arg = FolloweeArg {
        followee_principal_id: alice_principal_id,
        followee_canister_id: alice_individual_template_canister_id,
    };
    let res = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "update_profiles_i_follow_toggle_list_with_specified_profile",
            encode_one(follow_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            bet_status.unwrap()
        })
        .unwrap();
    assert_eq!(res, true);

    // Bob follows Dan
    let follow_arg = FolloweeArg {
        followee_principal_id: dan_principal_id,
        followee_canister_id: dan_individual_template_canister_id,
    };
    let res = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "update_profiles_i_follow_toggle_list_with_specified_profile",
            encode_one(follow_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            bet_status.unwrap()
        })
        .unwrap();
    assert_eq!(res, true);

    // Dan follows Alice
    let follow_arg = FolloweeArg {
        followee_principal_id: alice_principal_id,
        followee_canister_id: alice_individual_template_canister_id,
    };
    let res = pic
        .update_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "update_profiles_i_follow_toggle_list_with_specified_profile",
            encode_one(follow_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            bet_status.unwrap()
        })
        .unwrap();
    assert_eq!(res, true);

    // Dan follows Bob
    let follow_arg = FolloweeArg {
        followee_principal_id: bob_principal_id,
        followee_canister_id: bob_individual_template_canister_id,
    };
    let res = pic
        .update_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "update_profiles_i_follow_toggle_list_with_specified_profile",
            encode_one(follow_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            bet_status.unwrap()
        })
        .unwrap();
    assert_eq!(res, true);

    // Save snapshot
    let reclaim_principal_id = Principal::from_text(RECLAIM_CANISTER_PRINCIPAL_ID).unwrap();

<<<<<<< HEAD
    let alice_snap_len = pic
=======
    let res = pic
>>>>>>> 18666a8... fix for key str
        .update_call(
            alice_individual_template_canister_id,
            reclaim_principal_id,
            "save_snapshot_json",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: u32 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            bet_status
        })
        .unwrap();
    println!("save_snapshot_json len: {:?}", res);
<<<<<<< HEAD

    let bob_snap_len = pic
        .update_call(
            bob_individual_template_canister_id,
            reclaim_principal_id,
            "save_snapshot_json",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: u32 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            bet_status
        })
        .unwrap();
    println!("save_snapshot_json len: {:?}", res);

    let dan_snap_len = pic
        .update_call(
            dan_individual_template_canister_id,
            reclaim_principal_id,
            "save_snapshot_json",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: u32 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            bet_status
        })
        .unwrap();
    println!("save_snapshot_json len: {:?}", res);

    // Download Snapshots
    //
    let mut alice_snapshot = vec![];
    let CHUNK_SIZE = 50;
    let num_iters = (alice_snap_len as f64 / CHUNK_SIZE as f64).ceil() as u32;
    for i in 0..num_iters {
        let start = i * CHUNK_SIZE;
        let mut end = (i + 1) * CHUNK_SIZE;
        if end > alice_snap_len {
            end = alice_snap_len;
        }

        let chunk = pic
            .update_call(
                alice_individual_template_canister_id,
                reclaim_principal_id,
                "download_snapshot",
                candid::encode_args((start as u64, (end - start) as u64)).unwrap(),
            )
            .map(|reply_payload| {
                let payload: Vec<u8> = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
                payload
            })
            .unwrap();
        alice_snapshot.extend(chunk);
    }

    let bob_snapshot = pic
        .update_call(
            bob_individual_template_canister_id,
            reclaim_principal_id,
            "download_snapshot",
            candid::encode_args((0 as u64, bob_snap_len as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: Vec<u8> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let dan_snapshot = pic
        .update_call(
            dan_individual_template_canister_id,
            reclaim_principal_id,
            "download_snapshot",
            candid::encode_args((0 as u64, dan_snap_len as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: Vec<u8> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    // Clear snapshot

    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            reclaim_principal_id,
            "clear_snapshot",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            reclaim_principal_id,
            "download_snapshot",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: Vec<u8> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        });
    assert_eq!(res.is_err(), true);

    // Query Alice canister for info

    let fres1 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice profile: {:?}", fres1);

    let fres2 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_posts_of_this_user_profile_with_pagination",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile failed\n"),
                };
            profile.unwrap()
        })
        .unwrap();
    println!("Alice posts: {:?}", fres2);

    let fres3 = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_hot_or_not_bets_placed_by_this_profile_with_pagination",
            candid::encode_args((0 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Vec<PlacedBetDetail> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob hot or not bets: {:?}", fres3);

    let fres4 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_hot_or_not_bet_details_for_this_post",
            candid::encode_args((0 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: BettingStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice hot or not status for post: {:?}", fres4);

    let fres5 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_hot_or_not_bet_details_for_this_post",
            candid::encode_args((1 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: BettingStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice hot or not status for post: {:?}", fres5);

    let fres6 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_principals_this_profile_follows_paginated",
            candid::encode_args((0 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Vec<(FollowEntryId, FollowEntryDetail)> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice follows: {:?}", fres6);

    let fres7 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_principals_that_follow_this_profile_paginated",
            candid::encode_args((0 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Vec<(FollowEntryId, FollowEntryDetail)> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice followed by: {:?}", fres7);

    let fres8 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<
                Vec<(u64, TokenEvent)>,
                GetUserUtilityTokenTransactionHistoryError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile.unwrap()
        })
        .unwrap();
    println!("Alice token history: {:?}", fres8);

    let fres9 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice token balance: {:?}", fres9);

    let fres10 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_well_known_principal_value",
            candid::encode_args((KnownPrincipalType::CanisterIdPostCache,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Option<Principal> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice post cache prinicpal: {:?}", fres10);

    // Stop canisters

    let res = match pic.stop_canister(alice_individual_template_canister_id, None) {
        Ok(_) => println!("Alice stopped"),
        Err(e) => println!("Alice stop error: {:?}", e),
    };

    let res = match pic.stop_canister(bob_individual_template_canister_id, None) {
        Ok(_) => println!("Bob stopped"),
        Err(e) => println!("Bob stop error: {:?}", e),
    };

    let res = match pic.stop_canister(dan_individual_template_canister_id, None) {
        Ok(_) => println!("Dan stopped"),
        Err(e) => println!("Dan stop error: {:?}", e),
    };

    // Init 2nd gen canisters
    /// Alice 2
    let alice2_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(alice2_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(alice_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        alice2_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    /// Bob 2
    let bob2_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(bob2_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(bob_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        bob2_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    /// Dan 2
    let dan2_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(dan2_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(dan_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        dan2_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Check new canisters

    let res = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_posts_of_this_user_profile_with_pagination",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile failed\n"),
                };
            profile
        })
        .unwrap();
    println!("Alice 2 posts: {:?}", res);

    let res = pic
        .query_call(
            bob2_individual_template_canister_id,
            bob_principal_id,
            "get_hot_or_not_bets_placed_by_this_profile_with_pagination",
            candid::encode_args((0 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Vec<PlacedBetDetail> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob 2 hot or not bets: {:?}", res);

    // Restore state

    let res = pic
        .update_call(
            alice2_individual_template_canister_id,
            reclaim_principal_id,
            "receive_and_save_snaphot",
            candid::encode_args((0 as u64, alice_snapshot)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let res = pic
        .update_call(
            bob2_individual_template_canister_id,
            reclaim_principal_id,
            "receive_and_save_snaphot",
            candid::encode_args((0 as u64, bob_snapshot)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let res = pic
        .update_call(
            dan2_individual_template_canister_id,
            reclaim_principal_id,
            "receive_and_save_snaphot",
            candid::encode_args((0 as u64, dan_snapshot)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    // Load snapshots

    let res = pic
        .update_call(
            alice2_individual_template_canister_id,
            reclaim_principal_id,
            "load_snapshot",
            candid::encode_args((alice_snap_len as u64,)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let res = pic
        .update_call(
            bob2_individual_template_canister_id,
            reclaim_principal_id,
            "load_snapshot",
            candid::encode_args((bob_snap_len as u64,)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let res = pic
        .update_call(
            dan2_individual_template_canister_id,
            reclaim_principal_id,
            "load_snapshot",
            candid::encode_args((dan_snap_len as u64,)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    // Query Alice canister for info

    let fres1_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice profile: {:?}", fres1_1);
    assert_eq!(fres1_1, fres1);

    let fres2_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_posts_of_this_user_profile_with_pagination",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile failed\n"),
                };
            profile.unwrap()
        })
        .unwrap();
    println!("Alice posts: {:?}", fres2_1);
    assert_eq!(fres2_1, fres2);

    let fres3_1 = pic
        .query_call(
            bob2_individual_template_canister_id,
            bob_principal_id,
            "get_hot_or_not_bets_placed_by_this_profile_with_pagination",
            candid::encode_args((0 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Vec<PlacedBetDetail> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob hot or not bets: {:?}", fres3_1);
    assert_eq!(fres3_1, fres3);

    let fres4_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_hot_or_not_bet_details_for_this_post",
            candid::encode_args((0 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: BettingStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice hot or not status for post: {:?}", fres4_1);
    assert_eq!(fres4_1, fres4);

    let fres5_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_hot_or_not_bet_details_for_this_post",
            candid::encode_args((1 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: BettingStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice hot or not status for post: {:?}", fres5_1);
    assert_eq!(fres5_1, fres5);

    let fres6_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_principals_this_profile_follows_paginated",
            candid::encode_args((0 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Vec<(FollowEntryId, FollowEntryDetail)> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice follows: {:?}", fres6_1);
    assert_eq!(fres6_1, fres6);

    let fres7_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_principals_that_follow_this_profile_paginated",
            candid::encode_args((0 as usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Vec<(FollowEntryId, FollowEntryDetail)> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice followed by: {:?}", fres7_1);
    assert_eq!(fres7_1, fres7);

    let fres8_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<
                Vec<(u64, TokenEvent)>,
                GetUserUtilityTokenTransactionHistoryError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile.unwrap()
        })
        .unwrap();
    println!("Alice token history: {:?}", fres8_1);
    assert_eq!(fres8_1, fres8);

    let fres9_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice token balance: {:?}", fres9_1);
    assert_eq!(fres9_1, fres9);

    let fres10_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice_principal_id,
            "get_well_known_principal_value",
            candid::encode_args((KnownPrincipalType::CanisterIdPostCache,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Option<Principal> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice post cache prinicpal: {:?}", fres10_1);
    assert_eq!(fres10_1, fres10);
=======
>>>>>>> 18666a8... fix for key str
}

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}
