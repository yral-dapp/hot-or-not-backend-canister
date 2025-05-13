use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::platform_orchestrator::types::args::UpgradeCanisterArg,
    common::types::{known_principal::KnownPrincipalType, wasm::WasmType},
};
use test_utils::setup::{
    env::{
        pocket_ic_env::get_new_pocket_ic_env,
        pocket_ic_init::get_initialized_env_with_provisioned_known_canisters,
    },
    test_constants::get_mock_user_alice_principal_id,
};

#[test]
fn test_upgrading_specific_individual_canister_with_version() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();

    let platform_orchestrator = known_principal_map
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .copied()
        .unwrap();

    let known_principal_map =
        get_initialized_env_with_provisioned_known_canisters(&pocket_ic, known_principal_map);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .copied()
        .unwrap();

    let global_admin_principal = known_principal_map
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .copied()
        .unwrap();

    let alice_principal_id = get_mock_user_alice_principal_id();

    let alice_canister_id = pocket_ic
        .update_call(
            user_index_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let alice_canister_id: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            alice_canister_id
        })
        .unwrap()
        .unwrap();

    let individual_user_template_wasm_module = include_bytes!(
        "../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
    );

    pocket_ic
        .update_call(
            platform_orchestrator,
            global_admin_principal,
            "upgrade_canisters_in_network",
            candid::encode_one(UpgradeCanisterArg {
                canister: WasmType::IndividualUserWasm,
                version: "v2.0.0".to_string(),
                wasm_blob: individual_user_template_wasm_module.to_vec(),
            })
            .unwrap(),
        )
        .map(|res| {
            let res: Result<String, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet available capacity call failed"),
            };
            res
        })
        .unwrap()
        .unwrap();

    for _ in 0..100 {
        pocket_ic.tick();
    }

    pocket_ic
        .update_call(
            platform_orchestrator,
            global_admin_principal,
            "upgrade_canisters_in_network",
            candid::encode_one(UpgradeCanisterArg {
                canister: WasmType::IndividualUserWasm,
                version: "v2.1.0".to_string(),
                wasm_blob: individual_user_template_wasm_module.to_vec(),
            })
            .unwrap(),
        )
        .map(|res| {
            let res: Result<String, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet available capacity call failed"),
            };
            res
        })
        .unwrap()
        .unwrap();

    for _ in 0..100 {
        pocket_ic.tick();
    }

    let alice_canister_version = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_version",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let version: String = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                e => panic!("\n🛑 get_utility_token_balance failed\n {e:?}"),
            };
            version
        })
        .unwrap();

    assert_eq!(alice_canister_version, "v2.1.0");

    pocket_ic
        .update_call(
            platform_orchestrator,
            global_admin_principal,
            "upgrade_specific_individual_canister_with_version",
            candid::encode_args((alice_canister_id, "v2.0.0")).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<(), String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                e => {
                    panic!("\n🛑 upgrade_specific_individual_canister_with_version failed\n {e:?}")
                }
            };
            result
        })
        .unwrap()
        .unwrap();

    let alice_canister_version = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_version",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let version: String = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                e => panic!("\n🛑 get_utility_token_balance failed\n {e:?}"),
            };
            version
        })
        .unwrap();

    assert_eq!(alice_canister_version, "v2.0.0");
}
