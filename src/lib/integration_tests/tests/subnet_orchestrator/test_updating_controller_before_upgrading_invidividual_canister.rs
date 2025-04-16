use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::user_index::types::UpgradeStatus,
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{get_mock_user_alice_canister_id, get_mock_user_charlie_principal_id},
};

#[ignore]
#[test]
fn test_updating_controller_before_upgrading_invidividual_canister() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let charlie_global_admin = get_mock_user_charlie_principal_id();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "add_principal_as_global_admin",
            candid::encode_one(charlie_global_admin).unwrap(),
        )
        .unwrap();

    let subnet_orchestrator_canister_id: Principal = pocket_ic
        .update_call(
            platform_canister_id,
            charlie_global_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[1]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for _ in 0..110 {
        pocket_ic.tick()
    }

    let alice_yral_principal_id = get_mock_user_alice_canister_id();

    let alice_yral_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id,
            alice_yral_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let canister_id: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id
        })
        .unwrap()
        .unwrap();

    let available_canister_list = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_list_of_available_canisters",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let caniter_ids: Vec<Principal> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Get list of availabe cansters call failed"),
            };
            caniter_ids
        })
        .unwrap();

    let available_canister_id = available_canister_list[0];

    pocket_ic
        .set_controllers(
            available_canister_id,
            Some(subnet_orchestrator_canister_id),
            vec![alice_yral_canister_id],
        )
        .unwrap();

    let individual_canister_wasm = include_bytes!(
        "../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
    );

    pocket_ic
        .update_call(
            subnet_orchestrator_canister_id,
            platform_canister_id,
            "start_upgrades_for_individual_canisters",
            candid::encode_args(("v1.1.0".to_owned(), individual_canister_wasm.to_vec())).unwrap(),
        )
        .map(|res| {
            let result: String = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("start upgrades for individual canister failed"),
            };
            result
        })
        .unwrap();

    for _ in 0..110 {
        pocket_ic.tick()
    }

    //Check version Installed
    let last_upgrade_status: UpgradeStatus = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_index_details_last_upgrade_status",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let upgrade_status: UpgradeStatus = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            upgrade_status
        })
        .unwrap();

    assert!(last_upgrade_status.version.eq("v1.1.0"));
    assert_eq!(last_upgrade_status.failed_canister_ids.len(), 0);
}
