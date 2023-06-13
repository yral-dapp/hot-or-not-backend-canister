use candid::Principal;
use ic_state_machine_tests::{CanisterId, Cycles, PrincipalId};
use shared_utils::common::types::known_principal::KnownPrincipalType;
use std::{fs::File, io::Read, path::PathBuf};

pub mod v1;

pub const CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS: Cycles = Cycles::new(20_000_000_000_000); // 20T
pub const CANISTER_INITIAL_CYCLES_FOR_NON_SPAWNING_CANISTERS: Cycles =
    Cycles::new(2_000_000_000_000); // 2T

pub fn get_global_super_admin_principal_id() -> PrincipalId {
    PrincipalId::new_self_authenticating(&[0])
}

pub fn get_global_super_admin_principal_id_v1() -> Principal {
    PrincipalId::new_self_authenticating(&[0]).0
}

pub fn get_alice_principal_id() -> PrincipalId {
    PrincipalId::new_self_authenticating(&[1])
}

pub fn get_mock_user_alice_principal_id() -> Principal {
    PrincipalId::new_self_authenticating(&[1]).0
}

pub fn get_bob_principal_id() -> PrincipalId {
    PrincipalId::new_self_authenticating(&[2])
}

pub fn get_mock_user_bob_principal_id() -> Principal {
    PrincipalId::new_self_authenticating(&[2]).0
}

pub fn get_charlie_principal_id() -> PrincipalId {
    PrincipalId::new_self_authenticating(&[3])
}

pub fn get_mock_user_charlie_principal_id() -> Principal {
    PrincipalId::new_self_authenticating(&[3]).0
}

pub fn get_dan_principal_id() -> PrincipalId {
    PrincipalId::new_self_authenticating(&[4])
}

pub fn get_mock_user_dan_principal_id() -> Principal {
    PrincipalId::new_self_authenticating(&[4]).0
}

pub fn get_mock_canister_id_post_cache() -> Principal {
    CanisterId::from_u64(0).get().0
}

pub fn get_mock_canister_id_root() -> Principal {
    CanisterId::from_u64(2).get().0
}

pub fn get_mock_canister_id_sns() -> Principal {
    CanisterId::from_u64(3).get().0
}

pub fn get_mock_canister_id_topic_cache() -> Principal {
    CanisterId::from_u64(4).get().0
}

pub fn get_mock_canister_id_user_index() -> Principal {
    CanisterId::from_u64(5).get().0
}

pub fn get_mock_canister_id_configuration() -> Principal {
    CanisterId::from_u64(6).get().0
}

pub fn get_mock_canister_id_data_backup() -> Principal {
    CanisterId::from_u64(7).get().0
}

pub fn get_mock_user_alice_canister_id() -> Principal {
    CanisterId::from_u64(8).get().0
}

pub fn get_mock_user_bob_canister_id() -> Principal {
    CanisterId::from_u64(9).get().0
}

pub fn get_mock_user_charlie_canister_id() -> Principal {
    CanisterId::from_u64(10).get().0
}

pub fn get_mock_user_dan_canister_id() -> Principal {
    CanisterId::from_u64(11).get().0
}

pub fn get_user_index_canister_wasm() -> Vec<u8> {
    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );
    file_path.push("../../../../target/wasm32-unknown-unknown/release/user_index.wasm.gz");

    let mut file = File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}

pub fn get_configuration_canister_wasm() -> Vec<u8> {
    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );
    file_path.push("../../../../target/wasm32-unknown-unknown/release/configuration.wasm.gz");

    let mut file = File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}

pub fn get_post_cache_canister_wasm() -> Vec<u8> {
    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );
    file_path.push("../../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz");

    let mut file = File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");
    bytes
}

pub fn get_canister_wasm(canister_type: KnownPrincipalType) -> Vec<u8> {
    let mut bytes = Vec::new();

    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );
    file_path.push("../../../target/wasm32-unknown-unknown/release");

    match canister_type {
        KnownPrincipalType::CanisterIdConfiguration => {
            file_path.push("configuration.wasm");
            let mut file = File::open(&file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
            file.read_to_end(&mut bytes).expect("Failed to read file");
        }
        KnownPrincipalType::CanisterIdDataBackup => {
            file_path.push("data_backup.wasm");
            let mut file = File::open(&file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
            file.read_to_end(&mut bytes).expect("Failed to read file");
        }
        KnownPrincipalType::CanisterIdPostCache => {
            file_path.push("post_cache.wasm");
            let mut file = File::open(&file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
            file.read_to_end(&mut bytes).expect("Failed to read file");
        }
        KnownPrincipalType::CanisterIdUserIndex => {
            file_path.push("user_index.wasm");
            let mut file = File::open(&file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
            file.read_to_end(&mut bytes).expect("Failed to read file");
        }

        _ => panic!("Canister type not supported"),
    };
    bytes
}
