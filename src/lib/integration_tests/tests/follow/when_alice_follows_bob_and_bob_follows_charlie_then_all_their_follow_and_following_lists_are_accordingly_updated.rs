use candid::Principal;
use ic_test_state_machine_client::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::FolloweeArg,
        error::{FollowAnotherUserProfileError, GetFollowerOrFollowingPageError},
        follow::{FollowEntryDetail, FollowEntryId},
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
    test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
        get_mock_user_charlie_principal_id,
    },
};

#[test]
fn when_alice_follows_bob_and_bob_follows_charlie_then_all_their_follow_and_following_lists_are_accordingly_updated(
) {
    let state_machine = get_new_state_machine();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap()
        .clone();
    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let charlie_principal_id = get_mock_user_charlie_principal_id();

    let alice_canister_id = state_machine.update_call(
        user_index_canister_id,
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

    let bob_canister_id = state_machine.update_call(
        user_index_canister_id,
        bob_principal_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let bob_canister_id: Principal = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        bob_canister_id
    }).unwrap();

    let charlie_canister_id = state_machine.update_call(
        user_index_canister_id,
        charlie_principal_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let charlie_canister_id: Principal = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        charlie_canister_id
    }).unwrap();

    // Alice follows Bob
    let followee_arg = FolloweeArg {
        followee_principal_id: bob_principal_id,
        followee_canister_id: bob_canister_id,
    };

    let follow_status = state_machine
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "update_profiles_i_follow_toggle_list_with_specified_profile",
            candid::encode_one(followee_arg.clone()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 update_profiles_i_follow_toggle_list_with_specified_profile failed\n"
                ),
            };
            result.unwrap()
        })
        .unwrap();

    assert_eq!(follow_status, true);

    let follow_status = state_machine
        .query_call(
            alice_canister_id,
            alice_principal_id,
            "do_i_follow_this_user",
            candid::encode_one(followee_arg).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 do_i_follow_this_user failed\n"),
            };
            result.unwrap()
        })
        .unwrap();

    assert_eq!(follow_status, true);

    let followee_arg = FolloweeArg {
        followee_canister_id: charlie_canister_id,
        followee_principal_id: charlie_principal_id,
    };

    let follow_status = state_machine
        .query_call(
            alice_canister_id,
            alice_principal_id,
            "do_i_follow_this_user",
            candid::encode_one(followee_arg).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 do_i_follow_this_user failed\n"),
            };
            result.unwrap()
        })
        .unwrap();

    assert_eq!(follow_status, false);

    // Bob follows Charlie
    let followee_arg = FolloweeArg {
        followee_canister_id: charlie_canister_id,
        followee_principal_id: charlie_principal_id,
    };

    let follow_status = state_machine
        .update_call(
            bob_canister_id,
            bob_principal_id,
            "update_profiles_i_follow_toggle_list_with_specified_profile",
            candid::encode_one(followee_arg).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<bool, FollowAnotherUserProfileError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 update_profiles_i_follow_toggle_list_with_specified_profile failed\n"
                ),
            };
            result.unwrap()
        })
        .unwrap();

    assert_eq!(follow_status, true);

    // Alice's following list should contain Bob
    let alice_following_list = state_machine
        .query_call(
            alice_canister_id,
            alice_principal_id,
            "get_profiles_i_follow_paginated",
            candid::encode_one(None::<u64>).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<
                Vec<(FollowEntryId, FollowEntryDetail)>,
                GetFollowerOrFollowingPageError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profiles_i_follow_paginated failed\n"),
            };
            result.unwrap()
        })
        .unwrap();

    assert_eq!(alice_following_list.len(), 1);
    assert_eq!(alice_following_list[0].0, 0);
    assert_eq!(alice_following_list[0].1.principal_id, bob_principal_id);
    assert_eq!(alice_following_list[0].1.canister_id, bob_canister_id);

    // Bob's follower list should contain Alice
    let bob_follower_list = state_machine
        .query_call(
            bob_canister_id,
            bob_principal_id,
            "get_profiles_that_follow_me_paginated",
            candid::encode_one(None::<u64>).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<
                Vec<(FollowEntryId, FollowEntryDetail)>,
                GetFollowerOrFollowingPageError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profiles_that_follow_me_paginated failed\n"),
            };
            result.unwrap()
        })
        .unwrap();

    assert_eq!(bob_follower_list.len(), 1);
    assert_eq!(bob_follower_list[0].0, 0);
    assert_eq!(bob_follower_list[0].1.principal_id, alice_principal_id);
    assert_eq!(bob_follower_list[0].1.canister_id, alice_canister_id);

    // Bob's following list should contain Charlie
    let bob_following_list = state_machine
        .query_call(
            bob_canister_id,
            bob_principal_id,
            "get_profiles_i_follow_paginated",
            candid::encode_one(None::<u64>).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<
                Vec<(FollowEntryId, FollowEntryDetail)>,
                GetFollowerOrFollowingPageError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profiles_i_follow_paginated failed\n"),
            };
            result.unwrap()
        })
        .unwrap();

    assert_eq!(bob_following_list.len(), 1);
    assert_eq!(bob_following_list[0].0, 0);
    assert_eq!(bob_following_list[0].1.principal_id, charlie_principal_id);
    assert_eq!(bob_following_list[0].1.canister_id, charlie_canister_id);
}
