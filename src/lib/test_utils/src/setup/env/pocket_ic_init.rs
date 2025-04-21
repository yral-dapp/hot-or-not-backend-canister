use std::collections::HashMap;

use candid::Principal;
use pocket_ic::{management_canister::CanisterSettings, PocketIc};
use shared_utils::{
    access_control::UserAccessRole,
    canister_specific::{
        post_cache::types::arg::PostCacheInitArgs, user_index::types::args::UserIndexInitArgs,
    },
    common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType},
};

use crate::setup::test_constants::{
    get_canister_wasm, get_global_super_admin_principal_id, get_mock_canister_id_sns,
    v1::{
        CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS,
        CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    },
};

pub fn get_initialized_env_with_provisioned_known_canisters(
    pocket_ic: &PocketIc,
) -> KnownPrincipalMap {
    let canister_provisioner = |cycle_amount: u128| {
        let settings = Some(CanisterSettings {
            controllers: Some(vec![get_global_super_admin_principal_id()]),
            ..Default::default()
        });
        let canister_id = pocket_ic.create_canister_with_settings(
            Some(get_global_super_admin_principal_id()),
            settings,
        );
        pocket_ic.add_cycles(canister_id, cycle_amount);
        canister_id
    };

    // * Provision canisters
    let mut known_principal_map_with_all_canisters = KnownPrincipalMap::default();
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        get_global_super_admin_principal_id(),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdPostCache,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS),
    );
    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdUserIndex,
        canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS),
    );

    known_principal_map_with_all_canisters.insert(
        KnownPrincipalType::CanisterIdSnsGovernance,
        get_mock_canister_id_sns(),
    );

    // * Install canisters
    let canister_installer = |canister_id: Principal, wasm_module: Vec<u8>, arg: Vec<u8>| {
        pocket_ic.install_canister(
            canister_id,
            wasm_module,
            arg,
            Some(get_global_super_admin_principal_id()),
        );
    };

    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdPostCache)
            .unwrap(),
        get_canister_wasm(KnownPrincipalType::CanisterIdPostCache),
        candid::encode_one(PostCacheInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            ..Default::default()
        })
        .unwrap(),
    );

    let mut user_index_access_control_map = HashMap::new();
    user_index_access_control_map.insert(
        get_global_super_admin_principal_id(),
        vec![
            UserAccessRole::CanisterAdmin,
            UserAccessRole::CanisterController,
        ],
    );

    canister_installer(
        *known_principal_map_with_all_canisters
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .unwrap(),
        get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
        candid::encode_one(UserIndexInitArgs {
            known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
            access_control_map: Some(user_index_access_control_map),
            version: String::from("v1.0.0"),
        })
        .unwrap(),
    );

    let user_index_canister_id = known_principal_map_with_all_canisters
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();

    // * Provision individual user canisters
    pocket_ic.add_cycles(*user_index_canister_id, 10_000_000_000_000_000);
    let individual_user_template_wasm = include_bytes!(
        "../../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
    );
    pocket_ic
        .update_call(
            *user_index_canister_id,
            get_global_super_admin_principal_id(),
            "create_pool_of_individual_user_available_canisters",
            candid::encode_args(("v1.0.0", individual_user_template_wasm.to_vec())).unwrap(),
        )
        .expect("Failed to create pool of individual user canisters");

    for _ in 0..100 {
        pocket_ic.tick();
    }

    known_principal_map_with_all_canisters
}