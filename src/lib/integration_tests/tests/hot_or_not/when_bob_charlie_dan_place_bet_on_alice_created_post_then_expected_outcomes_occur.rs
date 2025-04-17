use std::time::{Duration, SystemTime};

use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BetDirection, BettingStatus},
        post::PostDetailsFromFrontend,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
        get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
    },
};

#[test]
fn when_bob_charlie_dan_place_bet_on_alice_created_post_then_expected_outcomes_occur() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();
    let user_index_canister_id: Principal = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .copied()
        .unwrap();
    let alice_principal_id: Principal = get_mock_user_alice_principal_id();
    let bob_principal_id: Principal = get_mock_user_bob_principal_id();
    let charlie_principal_id: Principal = get_mock_user_charlie_principal_id();
    let dan_principal_id: Principal = get_mock_user_dan_principal_id();

    let alice_canister_id: Principal = pocket_ic
        .update_call(
            user_index_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let result: Result<Principal, String> = candid::decode_one(&payload).unwrap();
                    result.unwrap()
                }
                _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"),
            }
        })
        .expect("Failed to call user_index_canister");

    let bob_canister_id: Principal = pocket_ic
        .update_call(
            user_index_canister_id,
            bob_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let result: Result<Principal, String> = candid::decode_one(&payload).unwrap();
                    result.unwrap()
                }
                _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"),
            }
        })
        .expect("Failed to call user_index_canister");

    let charlie_canister_id: Principal = pocket_ic
        .update_call(
            user_index_canister_id,
            charlie_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let result: Result<Principal, String> = candid::decode_one(&payload).unwrap();
                    result.unwrap()
                }
                _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"),
            }
        })
        .expect("Failed to call user_index_canister");

    let dan_canister_id: Principal = pocket_ic
        .update_call(
            user_index_canister_id,
            dan_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let result: Result<Principal, String> = candid::decode_one(&payload).unwrap();
                    result.unwrap()
                }
                _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"),
            }
        })
        .expect("Failed to call user_index_canister");

    let post_creation_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_438_993))
        .unwrap();
    pocket_ic.set_time(post_creation_time);

    let newly_created_post_id: u64 = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "add_post_v2",
            candid::encode_one(PostDetailsFromFrontend {
                description: "This is a fun video to watch".to_string(),
                hashtags: vec!["fun".to_string(), "video".to_string()],
                video_uid: "abcd#1234".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            })
            .unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let result: Result<u64, String> = candid::decode_one(&payload).unwrap();
                    result.unwrap()
                }
                _ => panic!("\n🛑 add_post_v2 failed\n"),
            }
        })
        .expect("Failed to add post");

    let bob_bet_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_447_993))
        .unwrap();
    pocket_ic.set_time(bob_bet_time);

    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_canister_id,
        post_id: newly_created_post_id,
        bet_amount: 50,
        bet_direction: BetDirection::Hot,
    };

    let bob_bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> = pocket_ic
        .update_call(
            bob_canister_id,
            bob_principal_id,
            "bet_on_currently_viewing_post",
            candid::encode_one(bob_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 bet_on_currently_viewing_post failed\n"),
            }
        })
        .expect("Failed to place bet");

    assert!(bob_bet_status.is_ok());
    assert_eq!(
        bob_bet_status.unwrap(),
        BettingStatus::BettingOpen {
            started_at: post_creation_time,
            number_of_participants: 1,
            ongoing_slot: 3,
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
        }
    );

    let charlie_bet_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_458_793))
        .unwrap();
    pocket_ic.set_time(charlie_bet_time);

    let charlie_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_canister_id,
        post_id: newly_created_post_id,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };

    let charlie_bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> = pocket_ic
        .update_call(
            charlie_canister_id,
            charlie_principal_id,
            "bet_on_currently_viewing_post",
            candid::encode_one(charlie_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 bet_on_currently_viewing_post failed\n"),
            }
        })
        .expect("Failed to place bet");

    assert!(charlie_bet_status.is_ok());
    assert_eq!(
        charlie_bet_status.unwrap(),
        BettingStatus::BettingOpen {
            started_at: post_creation_time,
            number_of_participants: 1,
            ongoing_slot: 6,
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
        }
    );

    let dan_bet_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_469_593))
        .unwrap();
    pocket_ic.set_time(dan_bet_time);

    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_canister_id,
        post_id: newly_created_post_id,
        bet_amount: 10,
        bet_direction: BetDirection::Hot,
    };

    let dan_bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> = pocket_ic
        .update_call(
            dan_canister_id,
            dan_principal_id,
            "bet_on_currently_viewing_post",
            candid::encode_one(dan_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 bet_on_currently_viewing_post failed\n"),
            }
        })
        .expect("Failed to place bet");

    assert!(dan_bet_status.is_ok());
    assert_eq!(
        dan_bet_status.unwrap(),
        BettingStatus::BettingOpen {
            started_at: post_creation_time,
            number_of_participants: 1,
            ongoing_slot: 9,
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
        }
    );
}