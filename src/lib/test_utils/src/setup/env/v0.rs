use std::collections::HashMap;

use candid::Principal;
use ic_state_machine_tests::{
    CanisterId, CanisterInstallMode, CanisterSettingsArgs, Cycles, PrincipalId, StateMachine,
};
use shared_utils::{
    access_control::UserAccessRole,
    canister_specific::{
        configuration::types::args::ConfigurationInitArgs,
        data_backup::types::args::DataBackupInitArgs, user_index::types::args::UserIndexInitArgs,
    },
    common::types::{
        init_args::PostCacheInitArgs,
        known_principal::{KnownPrincipalMap, KnownPrincipalType},
    },
};

use crate::setup::test_constants::{
    get_canister_wasm, get_global_super_admin_principal_id_v1,
    CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS,
    CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
};

pub fn get_initialized_env_with_provisioned_known_canisters(
    state_machine: &StateMachine,
) -> KnownPrincipalMap {
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
    let mut known_principal_map_with_all_canisters = KnownPrincipalMap::default();
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
    canister_installer(
        known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdDataBackup)
            .unwrap()
            .clone(),
        get_canister_wasm(KnownPrincipalType::CanisterIdDataBackup),
        candid::encode_one(DataBackupInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            ..Default::default()
        })
        .unwrap(),
    );
    canister_installer(
        known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdPostCache)
            .unwrap()
            .clone(),
        get_canister_wasm(KnownPrincipalType::CanisterIdPostCache),
        candid::encode_one(PostCacheInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
        })
        .unwrap(),
    );

    let mut user_index_access_control_map = HashMap::new();
    user_index_access_control_map.insert(
        get_global_super_admin_principal_id_v1(),
        vec![
            UserAccessRole::CanisterAdmin,
            UserAccessRole::CanisterController,
        ],
    );

    canister_installer(
        known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .unwrap()
            .clone(),
        get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
        candid::encode_one(UserIndexInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            access_control_map: Some(user_index_access_control_map),
            ..Default::default()
        })
        .unwrap(),
    );

    known_principal_map_with_all_canisters
}

pub fn get_canister_id_of_specific_type_from_principal_id_map(
    principal_id_map: &HashMap<KnownPrincipalType, Principal>,
    canister_type: KnownPrincipalType,
) -> CanisterId {
    CanisterId::new(PrincipalId(
        principal_id_map
            .get(&canister_type)
            .expect("Canister type not found in principal id map")
            .clone(),
    ))
    .expect("Canister id is invalid")
}

#[cfg(test)]
mod test {
    use crate::setup::test_constants::{
        get_mock_canister_id_configuration, get_mock_canister_id_data_backup,
    };

    use super::*;

    #[test]
    fn test_get_canister_id_of_specific_type_from_principal_id_map() {
        let mut principal_id_map = KnownPrincipalMap::default();
        principal_id_map.insert(
            KnownPrincipalType::CanisterIdConfiguration,
            get_mock_canister_id_configuration(),
        );
        principal_id_map.insert(
            KnownPrincipalType::CanisterIdDataBackup,
            get_mock_canister_id_data_backup(),
        );

        let canister_id = get_canister_id_of_specific_type_from_principal_id_map(
            &principal_id_map,
            KnownPrincipalType::CanisterIdConfiguration,
        );
        assert_eq!(
            canister_id,
            CanisterId::new(PrincipalId(get_mock_canister_id_configuration())).unwrap()
        );

        let canister_id = get_canister_id_of_specific_type_from_principal_id_map(
            &principal_id_map,
            KnownPrincipalType::CanisterIdDataBackup,
        );
        assert_eq!(
            canister_id,
            CanisterId::new(PrincipalId(get_mock_canister_id_data_backup())).unwrap()
        );
    }
}
