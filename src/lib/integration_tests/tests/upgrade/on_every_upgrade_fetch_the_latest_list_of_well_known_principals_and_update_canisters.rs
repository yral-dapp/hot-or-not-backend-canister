use std::time::Duration;

use candid::Principal;
use ic_cdk::api::management_canister::main::CanisterInstallMode;
use ic_test_state_machine_client::{CanisterSettings, WasmResult};
use shared_utils::{
    canister_specific::configuration::types::args::ConfigurationInitArgs,
    common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType},
};
use test_utils::setup::{
    env::v1::get_new_state_machine,
    test_constants::{
        get_canister_wasm, get_global_super_admin_principal_id,
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
            controllers: Some(vec![get_global_super_admin_principal_id()]),
            ..Default::default()
        });
        let canister_id = state_machine
            .create_canister_with_settings(settings, Some(get_global_super_admin_principal_id()));
        state_machine.add_cycles(canister_id, cycle_amount);
        canister_id
    };

    // * Provision canisters
    let mut known_principal_map_with_all_canisters = KnownPrincipalMap::default();
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        get_global_super_admin_principal_id(),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdConfiguration,
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
                    Some(get_global_super_admin_principal_id()),
                );
            }
            CanisterInstallMode::Upgrade(_) => {
                state_machine
                    .upgrade_canister(
                        canister_id,
                        wasm_module,
                        arg,
                        Some(get_global_super_admin_principal_id()),
                    )
                    .unwrap();
            }
            _ => {}
        }
    };

    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdConfiguration)
            .unwrap(),
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
        get_global_super_admin_principal_id(),
    );
    incomplete_known_principal_map.insert(
        KnownPrincipalType::CanisterIdConfiguration,
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdConfiguration)
            .unwrap(),
    );
    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdConfiguration)
            .unwrap(),
        get_canister_wasm(KnownPrincipalType::CanisterIdConfiguration),
        candid::encode_one(ConfigurationInitArgs {
            known_principal_ids: Some(incomplete_known_principal_map.clone()),
            ..Default::default()
        })
        .unwrap(),
        CanisterInstallMode::Upgrade(None),
    );

    let user_index_canister_id_from_configuration_canister: Option<Principal> = state_machine
        .query_call(
            *known_principal_map_with_all_canisters
                .get(&KnownPrincipalType::CanisterIdConfiguration)
                .unwrap(),
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
}
