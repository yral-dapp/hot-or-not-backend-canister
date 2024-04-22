use std::collections::HashMap;

use candid::{encode_args, encode_one};
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        post_cache::types::arg::PostCacheInitArgs, user_index::types::args::UserIndexInitArgs,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::test_constants::get_mock_user_charlie_principal_id;

const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";

const USER_INDEX_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/user_index.wasm.gz";

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn user_index_canister_wasm() -> Vec<u8> {
    std::fs::read(USER_INDEX_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}

#[test]
fn recycle_canisters_test() {
    let pic = PocketIc::new();
    let admin_principal_id = get_mock_user_charlie_principal_id();

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );
    known_prinicipal_values.insert(KnownPrincipalType::CanisterIdUserIndex, admin_principal_id);

    let post_cache_wasm_bytes = post_cache_canister_wasm();
    let post_cache_args = PostCacheInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        upgrade_version_number: Some(1),
        version: "1".to_string(),
    };
    let post_cache_args_bytes = encode_one(post_cache_args).unwrap();
    pic.install_canister(
        post_cache_canister_id,
        post_cache_wasm_bytes,
        post_cache_args_bytes,
        None,
    );

    let user_index_canister_id = pic.create_canister_with_settings(Some(admin_principal_id), None);
    pic.add_cycles(user_index_canister_id, 2_000_000_000_000_000);
    let user_index_wasm = user_index_canister_wasm();
    let user_index_args = UserIndexInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        access_control_map: None,
        version: "1".to_string(),
    };
    let user_index_args_bytes = encode_one(user_index_args).unwrap();
    pic.install_canister(
        user_index_canister_id,
        user_index_wasm,
        user_index_args_bytes,
        Some(admin_principal_id),
    );

    // Individual template canisters
    let individual_template_wasm_bytes = individual_template_canister_wasm();

    let res = pic
        .update_call(
            user_index_canister_id,
            admin_principal_id,
            "start_upgrades_for_individual_canisters",
            encode_args(("1".to_string(), individual_template_wasm_bytes)).unwrap(),
        )
        .map(|reply_payload| {
            let result: String = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            result
        })
        .unwrap();
}
