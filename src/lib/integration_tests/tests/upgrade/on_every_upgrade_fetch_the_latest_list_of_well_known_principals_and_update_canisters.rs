use std::time::Duration;

use candid::Principal;
use ic_cdk::api::management_canister::{main::CanisterInstallMode, provisional::CanisterSettings};
use ic_test_state_machine_client::WasmResult;
use shared_utils::{
    canister_specific::{
        configuration::types::args::ConfigurationInitArgs,
        data_backup::types::args::DataBackupInitArgs,
    },
    common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType},
};
use test_utils::setup::{
    env::v1::get_new_state_machine,
    test_constants::{
        get_canister_wasm, get_global_super_admin_principal_id_v1,
        v1::{
            CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS,
            CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
        },
    },
};

#[test]
fn on_every_upgrade_fetch_the_latest_list_of_well_known_principals_and_update_canisters() {
    let state_machine = get_new_state_machine();

    let canister_provisioner = |cycle_amount: u128| {
        let settings = Some(CanisterSettings {
            controllers: Some(vec![get_global_super_admin_principal_id_v1()]),
            ..Default::default()
        });
        let canister_id = state_machine.create_canister_with_settings(
            settings,
            Some(get_global_super_admin_principal_id_v1()),
        );
        state_machine.add_cycles(canister_id, cycle_amount);
        canister_id
    };

    // * Provision canisters
    let mut known_principal_map_with_all_canisters = KnownPrincipalMap::default();
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        get_global_super_admin_principal_id_v1(),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdConfiguration,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdDataBackup,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdPostCache,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdUserIndex,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS),
    );

    // * Install canisters
    let canister_installer = |canister_id: Principal,
                              wasm_module: Vec<u8>,
                              arg: Vec<u8>,
                              install_mode: CanisterInstallMode| {
        match install_mode {
            CanisterInstallMode::Install => {
                state_machine.install_canister(
                    canister_id,
                    wasm_module,
                    arg,
                    Some(get_global_super_admin_principal_id_v1()),
                );
            }
            CanisterInstallMode::Upgrade => {
                state_machine
                    .upgrade_canister(
                        canister_id,
                        wasm_module,
                        arg,
                        Some(get_global_super_admin_principal_id_v1()),
                    )
                    .unwrap();
            }
            _ => {}
        }
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
        CanisterInstallMode::Install,
    );

    let mut incomplete_known_principal_map = KnownPrincipalMap::default();
    incomplete_known_principal_map.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        get_global_super_admin_principal_id_v1(),
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
        CanisterInstallMode::Upgrade,
    );

    let user_index_canister_id_from_configuration_canister: Option<Principal> = state_machine
        .query_call(
            known_principal_map_with_all_canisters
                .get(&KnownPrincipalType::CanisterIdConfiguration)
                .unwrap()
                .clone(),
            Principal::anonymous(),
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
        CanisterInstallMode::Install,
    );

    let user_index_canister_id_from_data_backup_canister: Option<Principal> = state_machine
        .query_call(
            known_principal_map_with_all_canisters
                .get(&KnownPrincipalType::CanisterIdDataBackup)
                .unwrap()
                .clone(),
            Principal::anonymous(),
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
        .upgrade_canister(
            known_principal_map_with_all_canisters
                .get(&KnownPrincipalType::CanisterIdDataBackup)
                .unwrap()
                .clone(),
            get_canister_wasm(KnownPrincipalType::CanisterIdDataBackup),
            candid::encode_one(DataBackupInitArgs {
                ..Default::default()
            })
            .unwrap(),
            Some(get_global_super_admin_principal_id_v1()),
        )
        .ok();

    state_machine.advance_time(Duration::from_secs(1));
    state_machine.tick();

    let user_index_canister_id_from_data_backup_canister: Option<Principal> = state_machine
        .query_call(
            known_principal_map_with_all_canisters
                .get(&KnownPrincipalType::CanisterIdDataBackup)
                .unwrap()
                .clone(),
            Principal::anonymous(),
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
