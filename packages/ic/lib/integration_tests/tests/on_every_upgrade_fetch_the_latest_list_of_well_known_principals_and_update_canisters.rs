use std::time::Duration;

use candid::Principal;
use ic_state_machine_tests::{
    CanisterId, CanisterInstallMode, CanisterSettingsArgs, Cycles, PrincipalId, StateMachine,
    WasmResult,
};
use shared_utils::{
    canister_specific::{
        configuration::types::args::ConfigurationInitArgs,
        data_backup::types::args::DataBackupInitArgs,
    },
    common::types::known_principal::{KnownPrincipalMapV1, KnownPrincipalType},
};
use test_utils::setup::test_constants::{
    get_canister_wasm, get_global_super_admin_principal_id_v1,
    CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS,
    CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
};

#[test]
fn on_every_upgrade_fetch_the_latest_list_of_well_known_principals_and_update_canisters() {
    let state_machine = StateMachine::new();

    let canister_provisioner = |cycles: Cycles| {
        state_machine.create_canister_with_cycles(
            cycles,
            Some(CanisterSettingsArgs {
                controllers: Some(vec![PrincipalId(get_global_super_admin_principal_id_v1())]),
                ..Default::default()
            }),
        )
    };

    // * Provision canisters
    let mut known_principal_map_with_all_canisters = KnownPrincipalMapV1::default();
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        get_global_super_admin_principal_id_v1(),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdConfiguration,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS)
            .get()
            .0,
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdDataBackup,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS)
            .get()
            .0,
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdPostCache,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS)
            .get()
            .0,
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdUserIndex,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS)
            .get()
            .0,
    );

    // * Install canisters
    let canister_installer = |canister_id: Principal, canister_wasm: Vec<u8>, payload: Vec<u8>| {
        state_machine
            .install_wasm_in_mode(
                CanisterId::new(PrincipalId(canister_id)).unwrap(),
                CanisterInstallMode::Install,
                canister_wasm,
                payload,
            )
            .ok()
    };

    canister_installer(
        known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdConfiguration)
            .unwrap()
            .clone(),
        get_canister_wasm(KnownPrincipalType::CanisterIdConfiguration),
        candid::encode_one(ConfigurationInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            ..Default::default()
        })
        .unwrap(),
    );

    let mut incomplete_known_principal_map = KnownPrincipalMapV1::default();
    incomplete_known_principal_map.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .unwrap()
            .clone(),
    );
    incomplete_known_principal_map.insert(
        KnownPrincipalType::CanisterIdConfiguration,
        known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdConfiguration)
            .unwrap()
            .clone(),
    );

    canister_installer(
        known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdConfiguration)
            .unwrap()
            .clone(),
        get_canister_wasm(KnownPrincipalType::CanisterIdConfiguration),
        candid::encode_one(ConfigurationInitArgs {
            known_principal_ids: Some(incomplete_known_principal_map.clone()),
            ..Default::default()
        })
        .unwrap(),
    );

    let user_index_canister_id_from_configuration_canister: Option<Principal> = state_machine
        .query(
            CanisterId::new(PrincipalId(
                known_principal_map_with_all_canisters
                    .get(&KnownPrincipalType::CanisterIdConfiguration)
                    .unwrap()
                    .clone(),
            ))
            .unwrap(),
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::CanisterIdUserIndex).unwrap(),
        )
        .map(|reply_payload| {
            let user_index_canister_id_from_configuration_canister: Option<Principal> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_well_known_principal_value failed\n"),
                };
            user_index_canister_id_from_configuration_canister
        })
        .expect("🛑 Failed to query the user index canister id from the configuration canister");

    assert!(user_index_canister_id_from_configuration_canister.is_some());

    canister_installer(
        known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdDataBackup)
            .unwrap()
            .clone(),
        get_canister_wasm(KnownPrincipalType::CanisterIdDataBackup),
        candid::encode_one(DataBackupInitArgs {
            known_principal_ids: Some(incomplete_known_principal_map.clone()),
            ..Default::default()
        })
        .unwrap(),
    );

    let user_index_canister_id_from_data_backup_canister: Option<Principal> = state_machine
        .query(
            CanisterId::new(PrincipalId(
                known_principal_map_with_all_canisters
                    .get(&KnownPrincipalType::CanisterIdDataBackup)
                    .unwrap()
                    .clone(),
            ))
            .unwrap(),
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::CanisterIdUserIndex).unwrap(),
        )
        .map(|reply_payload| {
            let user_index_canister_id_from_configuration_canister: Option<Principal> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_well_known_principal_value failed\n"),
                };
            user_index_canister_id_from_configuration_canister
        })
        .expect("🛑 Failed to query the user index canister id from the configuration canister");

    // * Freshly installed data backup canister does not have canister id of user index canister
    assert!(user_index_canister_id_from_data_backup_canister.is_none());

    // * Upgrade data backup canister
    state_machine
        .install_wasm_in_mode(
            CanisterId::new(PrincipalId(
                known_principal_map_with_all_canisters
                    .get(&KnownPrincipalType::CanisterIdDataBackup)
                    .unwrap()
                    .clone(),
            ))
            .unwrap(),
            CanisterInstallMode::Upgrade,
            get_canister_wasm(KnownPrincipalType::CanisterIdDataBackup),
            candid::encode_one(DataBackupInitArgs {
                ..Default::default()
            })
            .unwrap(),
        )
        .ok();

    state_machine.advance_time(Duration::from_secs(1));
    state_machine.tick();

    let user_index_canister_id_from_data_backup_canister: Option<Principal> = state_machine
        .query(
            CanisterId::new(PrincipalId(
                known_principal_map_with_all_canisters
                    .get(&KnownPrincipalType::CanisterIdDataBackup)
                    .unwrap()
                    .clone(),
            ))
            .unwrap(),
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::CanisterIdUserIndex).unwrap(),
        )
        .map(|reply_payload| {
            let user_index_canister_id_from_configuration_canister: Option<Principal> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_well_known_principal_value failed\n"),
                };
            user_index_canister_id_from_configuration_canister
        })
        .expect("🛑 Failed to query the user index canister id from the configuration canister");

    // * Upgrade data backup canister should have fetched canister id of user index canister from configuration canister
    assert!(user_index_canister_id_from_data_backup_canister.is_some());
}
