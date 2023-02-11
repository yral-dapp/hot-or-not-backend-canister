use std::{env, path::Path};

use ic_test_state_machine_client::StateMachine;

/// The path to the state machine binary to run the tests with
pub static STATE_MACHINE_BINARY: &str = "../../ic-test-state-machine";

pub fn get_new_state_machine() -> StateMachine {
    let path = match env::var_os("STATE_MACHINE_BINARY") {
        None => STATE_MACHINE_BINARY.to_string(),
        Some(path) => path
            .clone()
            .into_string()
            .unwrap_or_else(|_| panic!("Invalid string path for {path:?}")),
    };

    if !Path::new(&path).exists() {
        println!("
        Could not find state machine binary to run canister integration tests.

        I looked for it at {:?}. You can specify another path with the environment variable STATE_MACHINE_BINARY (note that I run from {:?}).

        Run the following command to get the binary:
            curl -sLO https://download.dfinity.systems/ic/$commit/binaries/$platform/ic-test-state-machine.gz
            gzip -d ic-test-state-machine.gz
            chmod +x ic-test-state-machine
        where $commit can be read from `.ic-commit` and $platform is 'x86_64-linux' for Linux and 'x86_64-darwin' for Intel/rosetta-enabled Darwin.
        ", &path, &env::current_dir().map(|x| x.display().to_string()).unwrap_or_else(|_| "an unknown directory".to_string()));
    }

    StateMachine::new(&path, false)
}

// pub fn get_initialized_env_with_provisioned_known_canisters(
//     state_machine: &StateMachine,
// ) -> HashMap<KnownPrincipalType, Principal> {
//     let canister_provisioner = |cycles: Cycles| {
//         // state_machine.create_canister_with_cycles(
//         //     cycles,
//         //     Some(CanisterSettingsArgs {
//         //         controllers: Some(vec![PrincipalId(get_global_super_admin_principal_id_v1())]),
//         //         ..Default::default()
//         //     }),
//         // )
//         state_machine.create_canister()
//     };

//     // * Provision canisters
//     let mut known_principal_map_with_all_canisters = HashMap<KnownPrincipalType, Principal>::default();
//     known_principal_map_with_all_canisters.insert(
//         KnownPrincipalType::UserIdGlobalSuperAdmin,
//         get_global_super_admin_principal_id_v1(),
//     );
//     known_principal_map_with_all_canisters.insert(
//         KnownPrincipalType::CanisterIdConfiguration,
//         canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS)
//             .get()
//             .0,
//     );
//     known_principal_map_with_all_canisters.insert(
//         KnownPrincipalType::CanisterIdDataBackup,
//         canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS)
//             .get()
//             .0,
//     );
//     known_principal_map_with_all_canisters.insert(
//         KnownPrincipalType::CanisterIdPostCache,
//         canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS)
//             .get()
//             .0,
//     );
//     known_principal_map_with_all_canisters.insert(
//         KnownPrincipalType::CanisterIdUserIndex,
//         canister_provisioner(CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS)
//             .get()
//             .0,
//     );

//     // * Install canisters
//     let canister_installer = |canister_id: Principal, canister_wasm: Vec<u8>, payload: Vec<u8>| {
//         state_machine
//             .install_wasm_in_mode(
//                 CanisterId::new(PrincipalId(canister_id)).unwrap(),
//                 CanisterInstallMode::Install,
//                 canister_wasm,
//                 payload,
//             )
//             .ok()
//     };

//     canister_installer(
//         known_principal_map_with_all_canisters
//             .get(&KnownPrincipalType::CanisterIdConfiguration)
//             .unwrap()
//             .clone(),
//         get_canister_wasm(KnownPrincipalType::CanisterIdConfiguration),
//         candid::encode_one(ConfigurationInitArgs {
//             known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
//             ..Default::default()
//         })
//         .unwrap(),
//     );
//     canister_installer(
//         known_principal_map_with_all_canisters
//             .get(&KnownPrincipalType::CanisterIdDataBackup)
//             .unwrap()
//             .clone(),
//         get_canister_wasm(KnownPrincipalType::CanisterIdDataBackup),
//         candid::encode_one(DataBackupInitArgs {
//             known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
//             ..Default::default()
//         })
//         .unwrap(),
//     );
//     canister_installer(
//         known_principal_map_with_all_canisters
//             .get(&KnownPrincipalType::CanisterIdPostCache)
//             .unwrap()
//             .clone(),
//         get_canister_wasm(KnownPrincipalType::CanisterIdPostCache),
//         candid::encode_one(PostCacheInitArgs {
//             known_principal_ids: known_principal_map_with_all_canisters.clone(),
//         })
//         .unwrap(),
//     );

//     let mut user_index_access_control_map = HashMap::new();
//     user_index_access_control_map.insert(
//         get_global_super_admin_principal_id_v1(),
//         vec![
//             UserAccessRole::CanisterAdmin,
//             UserAccessRole::CanisterController,
//         ],
//     );

//     canister_installer(
//         known_principal_map_with_all_canisters
//             .get(&KnownPrincipalType::CanisterIdUserIndex)
//             .unwrap()
//             .clone(),
//         get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
//         candid::encode_one(UserIndexInitArgs {
//             known_principal_ids: Some(known_principal_map_with_all_canisters.clone()),
//             access_control_map: Some(user_index_access_control_map),
//             ..Default::default()
//         })
//         .unwrap(),
//     );

//     known_principal_map_with_all_canisters
// }
