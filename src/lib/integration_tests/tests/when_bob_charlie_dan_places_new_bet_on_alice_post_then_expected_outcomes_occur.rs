use std::time::{Duration, SystemTime};

use candid::Principal;
use ic_state_machine_tests::{CanisterId, PrincipalId, StateMachine, WasmResult};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BetDirection, BettingStatus},
        post::{PostDetailsFromFrontend, PostViewDetailsFromFrontend},
    },
    common::types::{
        known_principal::KnownPrincipalType, top_posts::post_score_index_item::PostScoreIndexItem,
    },
    types::canister_specific::post_cache::error_types::TopPostsFetchError,
};
use test_utils::setup::{
    env::v0::{
        get_canister_id_of_specific_type_from_principal_id_map,
        get_initialized_env_with_provisioned_known_canisters,
    },
    test_constants::{
        get_alice_principal_id, get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
        get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
    },
};

#[test]
fn when_bob_charlie_dan_places_new_bet_on_alice_post_then_expected_outcomes_occur() {
    let state_machine = StateMachine::new();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = get_canister_id_of_specific_type_from_principal_id_map(
        &known_principal_map,
        KnownPrincipalType::CanisterIdUserIndex,
    );
    let post_cache_canister_id = get_canister_id_of_specific_type_from_principal_id_map(
        &known_principal_map,
        KnownPrincipalType::CanisterIdPostCache,
    );
    let alice_principal_id = get_alice_principal_id();

    println!("🧪 user_index_canister_id: {:?}", user_index_canister_id);

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

    println!("🧪 alice_canister_id: {:?}", alice_canister_id.to_text());

    let post_creation_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_438_993))
        .unwrap();
    state_machine.set_time(post_creation_time);

    // * Post is created by Alice
    let newly_created_post_id = state_machine
        .execute_ingress_as(
            alice_principal_id,
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "add_post_v2",
            candid::encode_args((PostDetailsFromFrontend {
                description: "This is a fun video to watch".to_string(),
                hashtags: vec!["fun".to_string(), "video".to_string()],
                video_uid: "abcd#1234".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post_v2 failed\n"),
            };
            assert!(result.is_ok());
            result.unwrap()
        })
        .unwrap();

    println!("🧪 newly_created_post_id: {:?}", newly_created_post_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 3000);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 3000);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let bob_event_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_447_993))
        .unwrap();
    state_machine.set_time(bob_event_time);

    // * Bob watches the video
    let bob_view_details = PostViewDetailsFromFrontend::WatchedMultipleTimes {
        watch_count: 4,
        percentage_watched: 23,
    };
    let result = state_machine.execute_ingress_as(
        PrincipalId(get_mock_user_bob_principal_id()),
        CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
        "update_post_add_view_details",
        candid::encode_args((returned_post.post_id, bob_view_details)).unwrap(),
    );

    assert!(result.is_ok());

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 4_840);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 4_840);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    // * Bob likes the post
    let like_status = state_machine
        .execute_ingress_as(
            PrincipalId(get_mock_user_bob_principal_id()),
            CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
            "update_post_toggle_like_status_by_caller",
            candid::encode_one(returned_post.post_id).unwrap(),
        )
        .map(|reply_payload| {
            let like_status: bool = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_post_toggle_like_status_by_caller failed\n"),
            };
            like_status
        })
        .unwrap();

    assert_eq!(like_status, true);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 6_840);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 6_840);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    // * Bob bets on the post
    let bob_place_bet_arg = PlaceBetArg {
        post_id: returned_post.post_id,
        bet_amount: 50,
        bet_direction: BetDirection::Hot,
    };

    let bet_status = state_machine
        .execute_ingress_as(
            PrincipalId(get_mock_user_bob_principal_id()),
            CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
            "bet_on_currently_viewing_post",
            candid::encode_one(bob_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 bet_on_currently_viewing_post failed\n"),
                };
            bet_status
        })
        .unwrap();
    assert!(bet_status.is_ok());
    assert_eq!(
        bet_status.unwrap(),
        BettingStatus::BettingOpen {
            started_at: post_creation_time,
            number_of_participants: 1,
            ongoing_slot: 3,
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
        }
    );

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 7_840);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 6_840);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let charlie_event_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_458_793))
        .unwrap();
    state_machine.set_time(charlie_event_time);

    // * Charlie watches the video
    let charlie_view_details = PostViewDetailsFromFrontend::WatchedMultipleTimes {
        watch_count: 7,
        percentage_watched: 97,
    };
    let result = state_machine.execute_ingress_as(
        PrincipalId(get_mock_user_charlie_principal_id()),
        CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
        "update_post_add_view_details",
        candid::encode_args((returned_post.post_id, charlie_view_details)).unwrap(),
    );

    assert!(result.is_ok());

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 6_549);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 5_549);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    // * Charlie bets on the post
    let charlie_place_bet_arg = PlaceBetArg {
        post_id: returned_post.post_id,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };

    let bet_status = state_machine
        .execute_ingress_as(
            PrincipalId(get_mock_user_charlie_principal_id()),
            CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
            "bet_on_currently_viewing_post",
            candid::encode_one(charlie_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 bet_on_currently_viewing_post failed\n"),
                };
            bet_status
        })
        .unwrap();
    assert!(bet_status.is_ok());
    assert_eq!(
        bet_status.unwrap(),
        BettingStatus::BettingOpen {
            started_at: post_creation_time,
            number_of_participants: 1,
            ongoing_slot: 6,
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
        }
    );

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 6_049);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 7_549);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let dan_event_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_469_593))
        .unwrap();
    state_machine.set_time(dan_event_time);

    // * Dan watches the video
    let dan_view_details = PostViewDetailsFromFrontend::WatchedMultipleTimes {
        watch_count: 2,
        percentage_watched: 11,
    };
    let result = state_machine.execute_ingress_as(
        PrincipalId(get_mock_user_dan_principal_id()),
        CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
        "update_post_add_view_details",
        candid::encode_args((returned_post.post_id, dan_view_details)).unwrap(),
    );

    assert!(result.is_ok());

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 5_642);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 7_142);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    // * Dan likes the post
    let like_status = state_machine
        .execute_ingress_as(
            PrincipalId(get_mock_user_dan_principal_id()),
            CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
            "update_post_toggle_like_status_by_caller",
            candid::encode_one(returned_post.post_id).unwrap(),
        )
        .map(|reply_payload| {
            let like_status: bool = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_post_toggle_like_status_by_caller failed\n"),
            };
            like_status
        })
        .unwrap();

    assert_eq!(like_status, true);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 6_267);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 7_767);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    // * Dan shares the post
    let incremented_share_count = state_machine
        .execute_ingress_as(
            PrincipalId(get_mock_user_dan_principal_id()),
            CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
            "update_post_increment_share_count",
            candid::encode_one(returned_post.post_id).unwrap(),
        )
        .map(|reply_payload| {
            let incremented_share_count: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_post_increment_share_count failed\n"),
            };
            incremented_share_count
        })
        .unwrap();

    assert_eq!(incremented_share_count, 1);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 12_517);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 14_017);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    // * Dan bets on the post
    let dan_place_bet_arg = PlaceBetArg {
        post_id: returned_post.post_id,
        bet_amount: 10,
        bet_direction: BetDirection::Hot,
    };

    let bet_status = state_machine
        .execute_ingress_as(
            PrincipalId(get_mock_user_dan_principal_id()),
            CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
            "bet_on_currently_viewing_post",
            candid::encode_one(dan_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 bet_on_currently_viewing_post failed\n"),
                };
            bet_status
        })
        .unwrap();
    assert!(bet_status.is_ok());
    assert_eq!(
        bet_status.unwrap(),
        BettingStatus::BettingOpen {
            started_at: post_creation_time,
            number_of_participants: 1,
            ongoing_slot: 9,
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
        }
    );

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 12_683);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 13_353);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let alice_event_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_510_993))
        .unwrap();
    state_machine.set_time(alice_event_time);

    // * Alice watches the video
    let alice_view_details = PostViewDetailsFromFrontend::WatchedMultipleTimes {
        watch_count: 1,
        percentage_watched: 5,
    };
    let result = state_machine.execute_ingress_as(
        PrincipalId(get_mock_user_alice_principal_id()),
        CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
        "update_post_add_view_details",
        candid::encode_args((returned_post.post_id, alice_view_details)).unwrap(),
    );

    assert!(result.is_ok());

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 9_810);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 10_480);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    // * Alice shares the post
    let incremented_share_count = state_machine
        .execute_ingress_as(
            PrincipalId(get_mock_user_alice_principal_id()),
            CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
            "update_post_increment_share_count",
            candid::encode_one(returned_post.post_id).unwrap(),
        )
        .map(|reply_payload| {
            let incremented_share_count: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_post_increment_share_count failed\n"),
            };
            incremented_share_count
        })
        .unwrap();

    assert_eq!(incremented_share_count, 2);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 15_366);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = state_machine
        .query(
            post_cache_canister_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64,10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 16_036);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    // let alice_first_post_detail = state_machine
    //     .query(
    //         CanisterId::new(PrincipalId(returned_post.publisher_canister_id)).unwrap(),
    //         "get_individual_post_details_by_id",
    //         candid::encode_args((returned_post.post_id,)).unwrap(),
    //     )
    //     .map(|reply_payload| {
    //         let post_details: PostDetailsForFrontend = match reply_payload {
    //             WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
    //             _ => panic!("\n🛑 get_individual_post_details_by_id failed\n"),
    //         };
    //         post_details
    //     })
    //     .unwrap();

    // println!("🧪 alice_first_post_detail: {:?}", alice_first_post_detail);
    // assert_eq!(alice_first_post_detail.home_feed_ranking_score, 7_840);
    // assert_eq!(
    //     alice_first_post_detail.hot_or_not_feed_ranking_score,
    //     Some(6_840)
    // );
    // assert_eq!(alice_first_post_detail.home_feed_ranking_score, 3000);
    // assert_eq!(
    //     alice_first_post_detail.hot_or_not_feed_ranking_score,
    //     Some(3000)
    // );
}
