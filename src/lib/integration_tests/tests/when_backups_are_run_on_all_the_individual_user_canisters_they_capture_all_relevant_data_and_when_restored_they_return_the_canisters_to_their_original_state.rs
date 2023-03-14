use candid::Principal;
use ic_state_machine_tests::{
    CanisterId, CanisterInstallMode, PrincipalId, StateMachine, WasmResult,
};
use shared_utils::{
    canister_specific::{
        data_backup::types::{all_user_data::AllUserData, backup_statistics::BackupStatistics},
        individual_user_template::types::{
            post::{PostDetailsForFrontend, PostDetailsFromFrontend},
            profile::UserProfileUpdateDetailsFromFrontend,
        },
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::v0::{
        get_canister_id_of_specific_type_from_principal_id_map,
        get_initialized_env_with_provisioned_known_canisters,
    },
    test_constants::{
        get_canister_wasm, get_global_super_admin_principal_id_v1,
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    },
};

#[ignore]
#[test]
fn when_backups_are_run_on_all_the_individual_user_canisters_they_capture_all_relevant_data_and_when_restored_they_return_the_canisters_to_their_original_state(
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
            "add_post",
            candid::encode_args((PostDetailsFromFrontend {
                description: "alice post 0 - description".to_string(),
                hashtags: vec!["alice-tag-0".to_string(), "alice-tag-1".to_string()],
                video_uid: "alice-video-0".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .map(|reply_payload| {
            let (newly_created_post_id,): (u64,) = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id
        })
        .unwrap();

    state_machine
        .execute_ingress_as(
            alice_principal_id,
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "add_post",
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
            "add_post",
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
            "add_post",
            candid::encode_args((PostDetailsFromFrontend {
                description: "bob post 1 - description".to_string(),
                hashtags: vec!["bob-tag-2".to_string(), "bob-tag-3".to_string()],
                video_uid: "bob-video-1".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .unwrap();

    // TODO: add a stress test case where we are adding 1000 posts and check if all are sent back

    state_machine
        .execute_ingress_as(
            bob_principal_id,
            CanisterId::new(PrincipalId(bob_canister_id)).unwrap(),
            "update_principals_i_follow_toggle_list_with_principal_specified",
            candid::encode_one(get_mock_user_alice_principal_id()).unwrap(),
        )
        .unwrap();

    state_machine
        .install_wasm_in_mode(
            user_index_canister_id,
            CanisterInstallMode::Upgrade,
            get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
            candid::encode_one(()).unwrap(),
        )
        .unwrap();

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

    state_machine
        .execute_ingress_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            user_index_canister_id,
            "update_user_index_upgrade_user_canisters_with_latest_wasm",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();

    state_machine
        .execute_ingress_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            user_index_canister_id,
            "backup_all_individual_user_canisters",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();

    state_machine
        .execute_ingress_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            user_index_canister_id,
            "update_user_index_upgrade_user_canisters_with_latest_wasm",
            candid::encode_one(()).unwrap(),
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

    assert_eq!(backup_statistics.number_of_user_entries, 2);

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

    println!("alice_backup_details = {:?}", alice_backup_details);

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

    let bob_backup_details = state_machine
        .query_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            data_backup_canister_id,
            "get_individual_users_backup_data_entry",
            candid::encode_one(bob_principal_id.0).unwrap(),
        )
        .map(|reply_payload| {
            let bob_backup_details: Option<AllUserData> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_individual_users_backup_data_entry failed\n"),
            };
            bob_backup_details
        })
        .unwrap()
        .unwrap();

    assert!(bob_backup_details.user_principal_id == bob_principal_id.0);
    assert!(bob_backup_details.user_canister_id == bob_canister_id);
    assert!(
        bob_backup_details
            .canister_data
            .profile
            .unique_user_name
            .unwrap()
            == bob_unique_username
    );
    assert!(bob_backup_details.canister_data.all_created_posts.len() == 2);
    let bob_post_0 = bob_backup_details
        .canister_data
        .all_created_posts
        .get(&0)
        .unwrap();
    assert!(bob_post_0.description == "bob post 0 - description");
    assert!(bob_post_0.hashtags == vec!["bob-tag-0".to_string(), "bob-tag-1".to_string()]);
    assert!(bob_post_0.video_uid == "bob-video-0");
    assert!(bob_post_0.creator_consent_for_inclusion_in_hot_or_not == true);
    let bob_post_1 = bob_backup_details
        .canister_data
        .all_created_posts
        .get(&1)
        .unwrap();
    assert!(bob_post_1.description == "bob post 1 - description");
    assert!(bob_post_1.hashtags == vec!["bob-tag-2".to_string(), "bob-tag-3".to_string()]);
    assert!(bob_post_1.video_uid == "bob-video-1");
    assert!(bob_post_1.creator_consent_for_inclusion_in_hot_or_not == true);
    let token_data = bob_backup_details.canister_data.token_data;
    assert_eq!(token_data.utility_token_balance, 1500);
    assert_eq!(token_data.utility_token_transaction_history_v1.len(), 2);
    assert_eq!(
        bob_backup_details.canister_data.principals_i_follow.len(),
        1
    );
    assert!(bob_backup_details
        .canister_data
        .principals_i_follow
        .contains(&get_mock_user_alice_principal_id()));
    assert_eq!(
        bob_backup_details
            .canister_data
            .profile
            .display_name
            .unwrap(),
        bob_display_name
    );
    assert_eq!(
        bob_backup_details
            .canister_data
            .profile
            .profile_picture_url
            .unwrap(),
        bob_profile_picture_url
    );

    // TODO: implement testing restore functionality after API migrated to heap versions
    // let mut user_index_access_control_map = HashMap::new();
    // user_index_access_control_map.insert(
    //     get_global_super_admin_principal_id_v1(),
    //     vec![
    //         UserAccessRole::CanisterAdmin,
    //         UserAccessRole::CanisterController,
    //     ],
    // );

    // state_machine
    //     .install_wasm_in_mode(
    //         user_index_canister_id,
    //         CanisterInstallMode::Reinstall,
    //         get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
    //         candid::encode_one(UserIndexInitArgs {
    //             known_principal_ids: Some(known_principal_map.clone()),
    //             access_control_map: Some(user_index_access_control_map),
    //             ..Default::default()
    //         })
    //         .unwrap(),
    //     )
    //     .unwrap();

    // let returned_principal = state_machine
    //     .query(
    //         user_index_canister_id,
    //         "get_user_canister_id_from_user_principal_id",
    //         candid::encode_one(alice_principal_id.0).unwrap(),
    //     )
    //     .map(|reply_payload| {
    //         let returned_principal: Option<Principal> = match reply_payload {
    //             WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
    //             _ => panic!("\n🛑 get_user_canister_id_from_user_principal_id failed\n"),
    //         };
    //         returned_principal
    //     })
    //     .unwrap();

    // assert_eq!(returned_principal, None);

    // state_machine
    //     .execute_ingress_as(
    //         PrincipalId(get_global_super_admin_principal_id_v1()),
    //         data_backup_canister_id,
    //         "send_restore_data_back_to_user_index_canister",
    //         candid::encode_one(()).unwrap(),
    //     )
    //     .unwrap();

    // state_machine.run_until_completion(10);

    // let returned_principal = state_machine
    //     .query(
    //         user_index_canister_id,
    //         "get_user_canister_id_from_user_principal_id",
    //         candid::encode_one(alice_principal_id.0).unwrap(),
    //     )
    //     .map(|reply_payload| {
    //         let returned_principal: Option<Principal> = match reply_payload {
    //             WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
    //             _ => panic!("\n🛑 get_user_canister_id_from_user_principal_id failed\n"),
    //         };
    //         returned_principal
    //     })
    //     .unwrap();

    // assert_eq!(returned_principal, Some(alice_canister_id));
}
