use std::collections::HashMap;

use candid::Principal;
use ic_state_machine_tests::{
    CanisterId, CanisterInstallMode, PrincipalId, StateMachine, WasmResult,
};
use shared_utils::{
    access_control::UserAccessRole,
    canister_specific::{
        data_backup::types::backup_statistics::BackupStatistics,
        user_index::types::args::UserIndexInitArgs,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env_v0::{
        get_canister_id_of_specific_type_from_principal_id_map,
        get_initialized_env_with_provisioned_known_canisters,
    },
    test_constants::{
        get_canister_wasm, get_global_super_admin_principal_id_v1,
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    },
};

#[test]
fn when_backups_are_run_on_the_user_index_canister_they_capture_all_relevant_data_and_when_restored_they_return_the_canister_to_its_original_state(
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
    let bob_principal_id = PrincipalId(get_mock_user_bob_principal_id());
    let alice_unique_username = "cool_alice_1234".to_string();
    let bob_unique_username = "hot_bob_1234".to_string();

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
            candid::encode_one(alice_unique_username).unwrap(),
        )
        .unwrap();

    let bob_canister_id = state_machine.execute_ingress_as(
      bob_principal_id,
      user_index_canister_id,
      "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
      candid::encode_one(()).unwrap(),
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
            candid::encode_one(bob_unique_username).unwrap(),
        )
        .unwrap();

    let returned_principal = state_machine
        .query(
            user_index_canister_id,
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::CanisterIdConfiguration).unwrap(),
        )
        .map(|reply_payload| {
            let returned_principal: Option<Principal> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_well_known_principal_value failed\n"),
            };
            returned_principal
        })
        .unwrap();

    println!(
        "🧪 Returned principal: {:?}",
        returned_principal.unwrap().to_text()
    );

    state_machine
        .execute_ingress_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            user_index_canister_id,
            "backup_data_to_backup_canister",
            candid::encode_one(()).unwrap(),
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
    println!("🧪 Installing WASM");

    println!("🧪 Data backup canister ID: {:?}", data_backup_canister_id);

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

    let mut user_index_access_control_map = HashMap::new();
    user_index_access_control_map.insert(
        get_global_super_admin_principal_id_v1(),
        vec![
            UserAccessRole::CanisterAdmin,
            UserAccessRole::CanisterController,
        ],
    );

    state_machine
        .install_wasm_in_mode(
            user_index_canister_id,
            CanisterInstallMode::Reinstall,
            get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
            candid::encode_one(UserIndexInitArgs {
                known_principal_ids: Some(known_principal_map.clone()),
                access_control_map: Some(user_index_access_control_map),
                ..Default::default()
            })
            .unwrap(),
        )
        .unwrap();

    let returned_principal = state_machine
        .query(
            user_index_canister_id,
            "get_user_canister_id_from_user_principal_id",
            candid::encode_one(alice_principal_id.0).unwrap(),
        )
        .map(|reply_payload| {
            let returned_principal: Option<Principal> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_canister_id_from_user_principal_id failed\n"),
            };
            returned_principal
        })
        .unwrap();

    assert_eq!(returned_principal, None);

    // TODO: restore data from backup canister
    state_machine
        .execute_ingress_as(
            PrincipalId(get_global_super_admin_principal_id_v1()),
            data_backup_canister_id,
            "send_restore_data_back_to_user_index_canister",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();

    state_machine.run_until_completion(10);

    // TODO: assert that newly installed user_index canister has user data for alice and bob
    let returned_principal = state_machine
        .query(
            user_index_canister_id,
            "get_user_canister_id_from_user_principal_id",
            candid::encode_one(alice_principal_id.0).unwrap(),
        )
        .map(|reply_payload| {
            let returned_principal: Option<Principal> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_canister_id_from_user_principal_id failed\n"),
            };
            returned_principal
        })
        .unwrap();

    assert_eq!(returned_principal, Some(alice_canister_id));
}
