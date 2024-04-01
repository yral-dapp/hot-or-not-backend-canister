use ic_cdk::api::management_canister::main::{install_code, start_canister, stop_canister, CanisterIdRecord, CanisterInstallMode, InstallCodeArgument};
use ic_cdk_macros::update;
use candid::Principal;
use shared_utils::{canister_specific::post_cache::types::arg::PostCacheInitArgs, common::types::{known_principal::{KnownPrincipalMap, KnownPrincipalType}, wasm::WasmType}, constant::{GLOBAL_SUPER_ADMIN_USER_ID, YRAL_POST_CACHE_CANISTER_ID}};

use crate::{guard::is_caller::is_caller_global_admin_or_controller, CANISTER_DATA};


#[update(guard = "is_caller_global_admin_or_controller")]
async fn reinstall_yral_post_cache_canister() {
    let post_cache_canister_id = Principal::from_text(YRAL_POST_CACHE_CANISTER_ID).unwrap();
    let canister_wasm = CANISTER_DATA.with_borrow_mut(|canister_data| canister_data.wasms.get(&WasmType::PostCacheWasm)).unwrap();
    let mut known_principal_map = KnownPrincipalMap::new();
    known_principal_map.insert(KnownPrincipalType::CanisterIdPlatformOrchestrator, Principal::from_text("74zq4-iqaaa-aaaam-ab53a-cai").unwrap());
    known_principal_map.insert(KnownPrincipalType::CanisterIdPostCache, post_cache_canister_id);
    known_principal_map.insert(KnownPrincipalType::UserIdGlobalSuperAdmin, Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap());
    let post_cache_init_arg = PostCacheInitArgs {
        known_principal_ids: Some(known_principal_map),
        upgrade_version_number: None,
        version: canister_wasm.version
    };
    
    stop_canister(CanisterIdRecord {canister_id: post_cache_canister_id}).await.unwrap();

    install_code(InstallCodeArgument {
        mode: CanisterInstallMode::Reinstall,
        canister_id: post_cache_canister_id,
        wasm_module: canister_wasm.wasm_blob,
        arg: candid::encode_one(post_cache_init_arg).unwrap()
    })
    .await
    .unwrap();

    start_canister(CanisterIdRecord {
        canister_id: post_cache_canister_id
    })
    .await
    .unwrap();
}