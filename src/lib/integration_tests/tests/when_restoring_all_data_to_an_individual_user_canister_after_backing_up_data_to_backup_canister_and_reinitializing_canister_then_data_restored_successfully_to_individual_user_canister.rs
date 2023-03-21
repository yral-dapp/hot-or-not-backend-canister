use candid::Principal;
use ic_state_machine_tests::{
    CanisterId, CanisterInstallMode, PrincipalId, StateMachine, WasmResult,
};
use shared_utils::{
    canister_specific::{
        data_backup::types::{all_user_data::AllUserData, backup_statistics::BackupStatistics},
        individual_user_template::types::{
            error::{GetFollowerOrFollowingError, GetPostsOfUserProfileError},
            post::{PostDetailsForFrontend, PostDetailsFromFrontend},
            profile::{UserProfileDetailsForFrontend, UserProfileUpdateDetailsFromFrontend},
        },
    },
    common::types::known_principal::KnownPrincipalType,
    types::{
        canister_specific::individual_user_template::error_types::GetUserUtilityTokenTransactionHistoryError,
        utility_token::token_event::TokenEvent,
    },
};
use test_utils::setup::{
    env::v0::{
        get_canister_id_of_specific_type_from_principal_id_map,
        get_initialized_env_with_provisioned_known_canisters,
    },
    test_constants::{
        get_global_super_admin_principal_id_v1, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id,
    },
};

#[test]
fn when_restoring_all_data_to_an_individual_user_canister_after_backing_up_data_to_backup_canister_and_reinitializing_canister_then_data_restored_successfully_to_individual_user_canister(
) {
    // * Arrange
    let state_machine = StateMachine::new();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = get_canister_id_of_specific_type_from_principal_id_map(
        &known_principal_map,
        KnownPrincipalType::CanisterIdUserIndex,
    );
    let data_backup_canister_id = get_canister_id_of_specific_type_from_principal_id_map(
        &known_principal_map,
        KnownPrincipalType::CanisterIdDataBackup,
    );
    let alice_principal_id = PrincipalId(get_mock_user_alice_principal_id());
    let alice_unique_username = "cool_alice_1234".to_string();
    let alice_display_name = "Alice".to_string();
    let alice_profile_picture_url = "https://alice.com".to_string();
    let bob_principal_id = PrincipalId(get_mock_user_bob_principal_id());
    let bob_unique_username = "hot_bob_1234".to_string();
    let bob_display_name = "Bob".to_string();
    let bob_profile_picture_url = "https://bob.com".to_string();
    let charlie_principal_id = PrincipalId(get_mock_user_charlie_principal_id());

    // * Act
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

    state_machine
        .execute_ingress_as(
            alice_principal_id,
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "update_profile_set_unique_username_once",
            candid::encode_one(alice_unique_username.clone()).unwrap(),
        )
        .unwrap();

    state_machine
        .execute_ingress_as(
            alice_principal_id,
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "update_profile_display_details",
            candid::encode_one(UserProfileUpdateDetailsFromFrontend {
                display_name: Some(alice_display_name.clone()),
                profile_picture_url: Some(alice_profile_picture_url.clone()),
            })
            .unwrap(),
        )
        .unwrap();

    state_machine
        .execute_ingress_as(
            alice_principal_id,
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "add_post_v2",
            candid::encode_args((PostDetailsFromFrontend {
                description: "alice post 0 - description".to_string(),
                hashtags: vec!["alice-tag-0".to_string(), "alice-tag-1".to_string()],
                video_uid: "alice-video-0".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post_v2 failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    state_machine
        .execute_ingress_as(
            alice_principal_id,
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "add_post_v2",
            candid::encode_args((PostDetailsFromFrontend {
                description: "alice post 1 - description".to_string(),
                hashtags: vec!["alice-tag-2".to_string(), "alice-tag-3".to_string()],
                video_uid: "alice-video-1".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .unwrap();

    let bob_canister_id = state_machine.execute_ingress_as(
      bob_principal_id,
      user_index_canister_id,
      "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
      candid::encode_one(Some(get_mock_user_alice_principal_id())).unwrap(),
    ).map(|reply_payload| {
        let (bob_canister_id,): (Principal,) = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        bob_canister_id
    }).unwrap();

    state_machine
        .execute_ingress_as(
            bob_principal_id,
            CanisterId::new(PrincipalId(bob_canister_id)).unwrap(),
            "update_profile_set_unique_username_once",
            candid::encode_one(bob_unique_username.clone()).unwrap(),
        )
        .unwrap();

    state_machine
        .execute_ingress_as(
            bob_principal_id,
            CanisterId::new(PrincipalId(bob_canister_id)).unwrap(),
            "update_profile_display_details",
            candid::encode_one(UserProfileUpdateDetailsFromFrontend {
                display_name: Some(bob_display_name.clone()),
                profile_picture_url: Some(bob_profile_picture_url.clone()),
            })
            .unwrap(),
        )
        .unwrap();

    state_machine
        .execute_ingress_as(
            bob_principal_id,
            CanisterId::new(PrincipalId(bob_canister_id)).unwrap(),
            "add_post_v2",
            candid::encode_args((PostDetailsFromFrontend {
                description: "bob post 0 - description".to_string(),
                hashtags: vec!["bob-tag-0".to_string(), "bob-tag-1".to_string()],
                video_uid: "bob-video-0".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .unwrap();

    state_machine
        .execute_ingress_as(
            bob_principal_id,
            CanisterId::new(PrincipalId(bob_canister_id)).unwrap(),
            "add_post_v2",
            candid::encode_args((PostDetailsFromFrontend {
                description: "bob post 1 - description".to_string(),
                hashtags: vec!["bob-tag-2".to_string(), "bob-tag-3".to_string()],
                video_uid: "bob-video-1".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .unwrap();

    state_machine
        .execute_ingress_as(
            bob_principal_id,
            CanisterId::new(PrincipalId(bob_canister_id)).unwrap(),
            "update_principals_i_follow_toggle_list_with_principal_specified",
            candid::encode_one(get_mock_user_alice_principal_id()).unwrap(),
        )
        .unwrap();

    state_machine.execute_ingress_as(
            charlie_principal_id,
            user_index_canister_id,
            "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
            candid::encode_one(()).unwrap(),
          ).map(|reply_payload| {
              let (charlie_canister_id,): (Principal,) = match reply_payload {
                  WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
                  _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
              };
              charlie_canister_id
          }).unwrap();

    state_machine
        .execute_ingress_as(
            alice_principal_id,
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "update_principals_i_follow_toggle_list_with_principal_specified",
            candid::encode_one(get_mock_user_charlie_principal_id()).unwrap(),
        )
        .unwrap();

    state_machine
        .execute_ingress_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "backup_data_to_backup_canister",
            candid::encode_args((get_mock_user_alice_principal_id(), alice_canister_id)).unwrap(),
        )
        .unwrap();

    let backup_statistics = state_machine
        .query(
            data_backup_canister_id,
            "get_current_backup_statistics",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let backup_statistics: BackupStatistics = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_current_backup_statistics failed\n"),
            };
            backup_statistics
        })
        .unwrap();

    assert_eq!(backup_statistics.number_of_user_entries, 1);

    let alice_backup_details = state_machine
        .query_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            data_backup_canister_id,
            "get_individual_users_backup_data_entry",
            candid::encode_one(alice_principal_id.0).unwrap(),
        )
        .map(|reply_payload| {
            let alice_backup_details: Option<AllUserData> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_individual_users_backup_data_entry failed\n"),
            };
            alice_backup_details
        })
        .unwrap()
        .unwrap();

    println!("🧪 alice_backup_details = {:?}", alice_backup_details);

    assert!(alice_backup_details.user_principal_id == alice_principal_id.0);
    assert!(alice_backup_details.user_canister_id == alice_canister_id);
    assert!(
        alice_backup_details
            .canister_data
            .profile
            .unique_user_name
            .unwrap()
            == alice_unique_username
    );
    println!(
        "alice_backup_details.canister_data.all_created_posts.len() = {:?}",
        alice_backup_details.canister_data.all_created_posts.len()
    );
    assert!(alice_backup_details.canister_data.all_created_posts.len() == 2);
    let alice_post_0 = alice_backup_details
        .canister_data
        .all_created_posts
        .get(&0)
        .unwrap();
    assert!(alice_post_0.description == "alice post 0 - description");
    assert!(alice_post_0.hashtags == vec!["alice-tag-0".to_string(), "alice-tag-1".to_string()]);
    assert!(alice_post_0.video_uid == "alice-video-0");
    assert!(alice_post_0.creator_consent_for_inclusion_in_hot_or_not == true);
    let alice_post_1 = alice_backup_details
        .canister_data
        .all_created_posts
        .get(&1)
        .unwrap();
    assert!(alice_post_1.description == "alice post 1 - description");
    assert!(alice_post_1.hashtags == vec!["alice-tag-2".to_string(), "alice-tag-3".to_string()]);
    assert!(alice_post_1.video_uid == "alice-video-1");
    assert!(alice_post_1.creator_consent_for_inclusion_in_hot_or_not == true);
    let token_data = alice_backup_details.canister_data.token_data;
    assert_eq!(token_data.utility_token_balance, 1500);
    assert_eq!(token_data.utility_token_transaction_history_v1.len(), 2);
    assert_eq!(
        alice_backup_details
            .canister_data
            .principals_that_follow_me
            .len(),
        1
    );
    assert!(alice_backup_details
        .canister_data
        .principals_that_follow_me
        .contains(&get_mock_user_bob_principal_id()));
    assert_eq!(
        alice_backup_details
            .canister_data
            .profile
            .display_name
            .unwrap(),
        alice_display_name
    );
    assert_eq!(
        alice_backup_details
            .canister_data
            .profile
            .profile_picture_url
            .unwrap(),
        alice_profile_picture_url
    );

    let canister_upgrade_result = state_machine
        .execute_ingress_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            user_index_canister_id,
            "upgrade_specific_individual_user_canister_with_latest_wasm",
            candid::encode_args((
                get_mock_user_alice_principal_id(),
                alice_canister_id,
                Some(CanisterInstallMode::Reinstall),
            ))
            .unwrap(),
        )
        .map(|reply_payload| {
            let canister_upgrade_result: String = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_all_posts failed\n"),
            };
            canister_upgrade_result
        })
        .unwrap();

    println!("🧪 canister_upgrade_result = {:?}", canister_upgrade_result);

    let posts_response = state_machine
        .query(
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "get_posts_of_this_user_profile_with_pagination",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts_response: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_posts_of_this_user_profile_with_pagination failed\n"),
                };
            posts_response
        })
        .unwrap();

    assert!(posts_response.is_err());
    assert_eq!(
        posts_response.unwrap_err(),
        GetPostsOfUserProfileError::ReachedEndOfItemsList
    );

    let restore_operation_response = state_machine
        .execute_ingress_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            data_backup_canister_id,
            "restore_backed_up_data_to_individual_users_canister",
            candid::encode_one(get_mock_user_alice_principal_id()).unwrap(),
        )
        .map(|reply_payload| {
            let restore_operation_response: String = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => {
                    panic!("\n🛑 restore_backed_up_data_to_individual_users_canister failed\n")
                }
            };
            restore_operation_response
        })
        .unwrap();

    println!(
        "🧪 restore_operation_response = {:?}",
        restore_operation_response
    );
    assert_eq!(restore_operation_response, "Success".to_string());

    let alice_first_post_detail = state_machine
        .query(
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "get_individual_post_details_by_id",
            candid::encode_args((0 as u64,)).unwrap(),
        )
        .map(|reply_payload| {
            let post_details: PostDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_individual_post_details_by_id failed\n"),
            };
            post_details
        })
        .unwrap();

    assert_eq!(
        alice_first_post_detail.description,
        "alice post 0 - description"
    );
    assert_eq!(
        alice_first_post_detail.hashtags,
        vec!["alice-tag-0", "alice-tag-1"]
    );
    assert_eq!(alice_first_post_detail.video_uid, "alice-video-0");

    let alice_second_post_detail = state_machine
        .query(
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "get_individual_post_details_by_id",
            candid::encode_args((1 as u64,)).unwrap(),
        )
        .map(|reply_payload| {
            let post_details: PostDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_individual_post_details_by_id failed\n"),
            };
            post_details
        })
        .unwrap();

    assert_eq!(
        alice_second_post_detail.description,
        "alice post 1 - description"
    );
    assert_eq!(
        alice_second_post_detail.hashtags,
        vec!["alice-tag-2", "alice-tag-3"]
    );
    assert_eq!(alice_second_post_detail.video_uid, "alice-video-1");

    let utility_token_balance = state_machine
        .query(
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "get_utility_token_balance",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let utility_token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            utility_token_balance
        })
        .unwrap();

    assert_eq!(utility_token_balance, 1500);

    let utility_token_transaction_history = state_machine
        .query(
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let utility_token_transaction_history: Result<
                Vec<(u64, TokenEvent)>,
                GetUserUtilityTokenTransactionHistoryError,
            > = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"
                ),
            };
            utility_token_transaction_history
        })
        .unwrap();

    assert!(utility_token_transaction_history.is_ok());
    assert_eq!(utility_token_transaction_history.unwrap().len(), 2);

    let principals_i_follow = state_machine
        .query(
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "get_principals_i_follow_paginated",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let principals_i_follow: Result<Vec<Principal>, GetFollowerOrFollowingError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_principals_i_follow_paginated failed\n"),
                };
            principals_i_follow
        })
        .unwrap();

    assert!(principals_i_follow.is_ok());

    let principals_i_follow = principals_i_follow.unwrap();

    assert_eq!(principals_i_follow.len(), 1);
    assert_eq!(principals_i_follow[0], get_mock_user_charlie_principal_id());

    let principals_that_follow_me = state_machine
        .query(
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "get_principals_that_follow_me_paginated",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let principals_that_follow_me: Result<Vec<Principal>, GetFollowerOrFollowingError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_principals_that_follow_me_paginated failed\n"),
                };
            principals_that_follow_me
        })
        .unwrap();

    assert!(principals_that_follow_me.is_ok());

    let principals_that_follow_me = principals_that_follow_me.unwrap();

    assert_eq!(principals_that_follow_me.len(), 1);
    assert_eq!(
        principals_that_follow_me[0],
        get_mock_user_bob_principal_id()
    );

    let profile_details = state_machine
        .query(
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "get_profile_details",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile_details: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile_details failed\n"),
            };
            profile_details
        })
        .unwrap();

    assert_eq!(
        profile_details.principal_id,
        get_mock_user_alice_principal_id()
    );
    assert_eq!(
        profile_details.unique_user_name,
        Some("cool_alice_1234".to_string())
    );
    assert_eq!(profile_details.display_name, Some("Alice".to_string()));
    assert_eq!(
        profile_details.profile_picture_url,
        Some("https://alice.com".to_string())
    );
}
