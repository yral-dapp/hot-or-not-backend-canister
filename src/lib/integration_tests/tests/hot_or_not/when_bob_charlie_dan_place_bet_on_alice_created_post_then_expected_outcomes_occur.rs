use std::time::Duration;

use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::{individual_user_template::types::{
        arg::PlaceBetArg,
        error::{BetOnCurrentlyViewingPostError, GetPostsOfUserProfileError},
        hot_or_not::{BetDirection, BetOutcomeForBetMaker, BettingStatus},
        post::{PostDetailsForFrontend, PostDetailsFromFrontend},
    }, user_index::types::args::UserIndexInitArgs},
    common::types::{known_principal::KnownPrincipalType, utility_token::token_event::{HotOrNotOutcomePayoutEvent, StakeEvent, TokenEvent}}, types::canister_specific::individual_user_template::error_types::GetUserUtilityTokenTransactionHistoryError,
};
use test_utils::setup::{
    env::{pocket_ic_env::get_new_pocket_ic_env, pocket_ic_init::get_initialized_env_with_provisioned_known_canisters},
    test_constants::{
        get_canister_wasm, get_mock_user_alice_principal_id, get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id
    },
};

#[ignore]
#[test]
fn when_bob_charlie_dan_place_bet_on_alice_created_post_then_expected_outcomes_occur() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&pocket_ic, known_principal_map);
    let platform_orchanstrator_canister_id = *known_principal_map
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .unwrap();
    let user_index_canister_id = *known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();
    println!(
        "🧪 user_index_canister_id: {:?}",
        user_index_canister_id.to_text()
    );
    
    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();

    let alice_canister_id = pocket_ic
        .update_call(
            user_index_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let alice_canister_id: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            alice_canister_id
        })
        .unwrap()
        .unwrap();

    let bob_canister_id = pocket_ic
        .update_call(
            user_index_canister_id,
            bob_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let bob_canister_id: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => {
                    panic!(
                        "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                    )
                }
            };
            bob_canister_id
        })
        .unwrap()
        .unwrap();

    let charlie_canister_id = pocket_ic
        .update_call(
            user_index_canister_id,
            charlie_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let charlie_canister_id: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            charlie_canister_id
        })
        .unwrap()
        .unwrap();

    let dan_canister_id = pocket_ic
        .update_call(
            user_index_canister_id,
            dan_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let dan_canister_id: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            dan_canister_id
        })
        .unwrap()
        .unwrap();

    println!("🧪 alice_canister_id: {:?}", alice_canister_id.to_text());


    // * Post is created by Alice
    let newly_created_post_id = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "add_post_v2",
            candid::encode_args((PostDetailsFromFrontend {
                description: "This is a fun video to watch".to_string(),
                is_nsfw: false,
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

    let returned_posts = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_posts_of_this_user_profile_with_pagination_cursor",
            candid::encode_args((0_u64,10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            let returned_posts: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts_of_this_user_profile_with_pagination_cursor failed\n"),
            };
            returned_posts.unwrap()
        })
        .unwrap();

    assert_eq!(returned_posts.len(), 1);

    let returned_post = returned_posts.first().unwrap();
    let post_creation_time = returned_post.created_at;

    assert_eq!(returned_post.id, newly_created_post_id);
    let publisher_canister_id = alice_canister_id;
    // * Bob bets on the post
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: publisher_canister_id,
        post_id: returned_post.id,
        bet_amount: 50,
        bet_direction: BetDirection::Hot,
    };

    let bet_status = pocket_ic
        .update_call(
            bob_canister_id,
            get_mock_user_bob_principal_id(),
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
    println!("🧪 bet_status: {:?}, post_creation_time: {:?}", bet_status, post_creation_time);
    assert!(bet_status.is_ok());
    assert_eq!(
        bet_status.unwrap(),
        BettingStatus::BettingOpen {
            started_at: post_creation_time,
            number_of_participants: 1,
            ongoing_slot: 1,
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
        }
    );

    // * Charlie bets on the post
    let charlie_place_bet_arg = PlaceBetArg {
        post_canister_id: publisher_canister_id,
        post_id: returned_post.id,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };

    let bet_status = pocket_ic
        .update_call(
            charlie_canister_id,
            get_mock_user_charlie_principal_id(),
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
            number_of_participants: 2,
            ongoing_slot: 1,
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
        }
    );

    // * Dan bets on the post
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: publisher_canister_id,
        post_id: returned_post.id,
        bet_amount: 10,
        bet_direction: BetDirection::Hot,
    };

    let bet_status = pocket_ic
        .update_call(
            dan_canister_id,
            get_mock_user_dan_principal_id(),
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
            number_of_participants: 3,
            ongoing_slot: 1,
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
        }
    );

    // * Restart their canisters
    pocket_ic
        .upgrade_canister(
            user_index_canister_id,
            get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
            candid::encode_one(UserIndexInitArgs {
                ..Default::default()
            })
            .unwrap(),
            Some(platform_orchanstrator_canister_id),
        )
        .unwrap();

    // pocket_ic.advance_time(Duration::from_secs(30));
    // pocket_ic.tick();

    // * advance time to the end of the first slot and then 5 minutes
    pocket_ic.advance_time(Duration::from_secs(60 * (60 + 5)));
    pocket_ic.tick();

    // * Alice outcome
    let alice_token_balance = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let alice_token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            alice_token_balance
        })
        .unwrap();

    assert_eq!(alice_token_balance, 1000 + 16);

    let alice_token_transaction_history = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0_u64, 10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            let alice_token_transaction_history: Result<
                Vec<(u64, TokenEvent)>,
                GetUserUtilityTokenTransactionHistoryError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"
                ),
            };
            assert!(alice_token_transaction_history.is_ok());
            alice_token_transaction_history.unwrap()
        })
        .unwrap();

    assert_eq!(alice_token_transaction_history.len(), 2);
    assert_eq!(
        alice_token_transaction_history.first().unwrap().1,
        TokenEvent::HotOrNotOutcomePayout {
            amount: 16,
            details: HotOrNotOutcomePayoutEvent::CommissionFromHotOrNotBet {
                post_canister_id: publisher_canister_id,
                post_id: 0,
                slot_id: 1,
                room_id: 1,
                room_pot_total_amount: 160
            },
            timestamp: if let TokenEvent::HotOrNotOutcomePayout { timestamp, .. } =
                alice_token_transaction_history.first().unwrap().1.clone()
            {
                timestamp
            } else {
                panic!("\n🛑 unexpected token event\n");
            },
        }
    );

    // * Bob outcome
    let bob_token_balance = pocket_ic
        .query_call(
            bob_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let bob_token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            bob_token_balance
        })
        .unwrap();

    println!("🧪 bob_token_balance: {}", bob_token_balance);
    assert_eq!(bob_token_balance, 1000 - 50 + 90);

    let bob_token_transaction_history = pocket_ic
        .query_call(
            bob_canister_id,
            Principal::anonymous(),
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0_u64, 10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            let bob_token_transaction_history: Result<
                Vec<(u64, TokenEvent)>,
                GetUserUtilityTokenTransactionHistoryError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"
                ),
            };
            assert!(bob_token_transaction_history.is_ok());
            bob_token_transaction_history.unwrap()
        })
        .unwrap();

    println!(
        "🧪 bob_token_transaction_history: {:?}",
        bob_token_transaction_history
    );

    assert_eq!(bob_token_transaction_history.len(), 3);
    assert_eq!(
        bob_token_transaction_history.first().unwrap().1,
        TokenEvent::HotOrNotOutcomePayout {
            amount: 90,
            details: HotOrNotOutcomePayoutEvent::WinningsEarnedFromBet {
                post_canister_id: alice_canister_id,
                post_id: 0,
                slot_id: 1,
                room_id: 1,
                event_outcome: BetOutcomeForBetMaker::Won(90),
                winnings_amount: 90
            },
            timestamp: if let TokenEvent::HotOrNotOutcomePayout { timestamp, .. } =
                bob_token_transaction_history.first().unwrap().1.clone()
            {
                timestamp
            } else {
                panic!("\n🛑 unexpected token event\n");
            },
        }
    );
    assert_eq!(
        bob_token_transaction_history.get(1).unwrap().1,
        TokenEvent::Stake {
            amount: 50,
            details: StakeEvent::BetOnHotOrNotPost {
                post_canister_id: alice_canister_id,
                post_id: 0,
                bet_amount: 50,
                bet_direction: BetDirection::Hot
            },
            timestamp: if let TokenEvent::Stake { timestamp, .. } =
                bob_token_transaction_history.get(1).unwrap().1.clone()
            {
                timestamp
            } else {
                panic!("\n🛑 unexpected token event\n");
            }
        }
    );

    // * Charlie outcome
    let charlie_token_balance = pocket_ic
        .query_call(
            charlie_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let charlie_token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            charlie_token_balance
        })
        .unwrap();

    println!("🧪 charlie_token_balance: {}", charlie_token_balance);
    assert_eq!(charlie_token_balance, 1000 - 100);

    let charlie_token_transaction_history = pocket_ic
        .query_call(
            charlie_canister_id,
            Principal::anonymous(),
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0_u64, 10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            let charlie_token_transaction_history: Result<
                Vec<(u64, TokenEvent)>,
                GetUserUtilityTokenTransactionHistoryError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"
                ),
            };
            assert!(charlie_token_transaction_history.is_ok());
            charlie_token_transaction_history.unwrap()
        })
        .unwrap();

    println!(
        "🧪 charlie_token_transaction_history: {:?}",
        charlie_token_transaction_history
    );

    assert_eq!(charlie_token_transaction_history.len(), 3);
    assert_eq!(
        charlie_token_transaction_history.first().unwrap().1,
        TokenEvent::HotOrNotOutcomePayout {
            amount: 0,
            details: HotOrNotOutcomePayoutEvent::WinningsEarnedFromBet {
                post_canister_id: alice_canister_id,
                post_id: 0,
                slot_id: 1,
                room_id: 1,
                event_outcome: BetOutcomeForBetMaker::Lost,
                winnings_amount: 0
            },
            timestamp: if let TokenEvent::HotOrNotOutcomePayout { timestamp, .. } =
                charlie_token_transaction_history.first().unwrap().1.clone()
            {
                timestamp
            } else {
                panic!("\n🛑 unexpected token event\n");
            },
        }
    );
    assert_eq!(
        charlie_token_transaction_history.get(1).unwrap().1,
        TokenEvent::Stake {
            amount: 100,
            details: StakeEvent::BetOnHotOrNotPost {
                post_canister_id: alice_canister_id,
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Not
            },
            timestamp: if let TokenEvent::Stake { timestamp, .. } =
                charlie_token_transaction_history.get(1).unwrap().1.clone()
            {
                timestamp
            } else {
                panic!("\n🛑 unexpected token event\n");
            }
        }
    );

    // * Dan outcome
    let dan_token_balance = pocket_ic
        .query_call(
            dan_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let dan_token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            dan_token_balance
        })
        .unwrap();

    println!("🧪 dan_token_balance: {}", dan_token_balance);
    assert_eq!(dan_token_balance, 1000 - 10 + 18);

    let dan_token_transaction_history = pocket_ic
        .query_call(
            dan_canister_id,
            Principal::anonymous(),
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0_u64, 10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            let dan_token_transaction_history: Result<
                Vec<(u64, TokenEvent)>,
                GetUserUtilityTokenTransactionHistoryError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"
                ),
            };
            assert!(dan_token_transaction_history.is_ok());
            dan_token_transaction_history.unwrap()
        })
        .unwrap();

    println!(
        "🧪 dan_token_transaction_history: {:?}",
        dan_token_transaction_history
    );

    assert_eq!(dan_token_transaction_history.len(), 3);
    assert_eq!(
        dan_token_transaction_history.first().unwrap().1,
        TokenEvent::HotOrNotOutcomePayout {
            amount: 18,
            details: HotOrNotOutcomePayoutEvent::WinningsEarnedFromBet {
                post_canister_id: alice_canister_id,
                post_id: 0,
                slot_id: 1,
                room_id: 1,
                event_outcome: BetOutcomeForBetMaker::Won(18),
                winnings_amount: 18
            },
            timestamp: if let TokenEvent::HotOrNotOutcomePayout { timestamp, .. } =
                dan_token_transaction_history.first().unwrap().1.clone()
            {
                timestamp
            } else {
                panic!("\n🛑 unexpected token event\n");
            },
        }
    );
    assert_eq!(
        dan_token_transaction_history.get(1).unwrap().1,
        TokenEvent::Stake {
            amount: 10,
            details: StakeEvent::BetOnHotOrNotPost {
                post_canister_id: alice_canister_id,
                post_id: 0,
                bet_amount: 10,
                bet_direction: BetDirection::Hot
            },
            timestamp: if let TokenEvent::Stake { timestamp, .. } =
                dan_token_transaction_history.get(1).unwrap().1.clone()
            {
                timestamp
            } else {
                panic!("\n🛑 unexpected token event\n");
            }
        }
    );
}