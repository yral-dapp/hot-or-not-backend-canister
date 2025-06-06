use candid::{encode_one, Principal};
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cdao::DeployedCdaoCanisters, session::SessionType,
    },
    common::types::known_principal::{self, KnownPrincipalType},
    constant::SNS_WASM_W_PRINCIPAL_ID,
};
use test_utils::setup::{
    self,
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_mock_user_alice_principal_id,
        get_mock_user_charlie_principal_id,
    },
};

use crate::utils::{setup_default_sns_creator_token, setup_sns_w_canister_for_creator_dao};

#[test]
pub fn test_deletion_of_creator_tokens() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();

    let super_admin = *known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let charlie_global_admin = get_mock_user_charlie_principal_id();

    let platform_canister_id = *known_principal
        .get(&known_principal::KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .unwrap();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "add_principal_as_global_admin",
            candid::encode_one(charlie_global_admin).unwrap(),
        )
        .unwrap();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "update_global_known_principal",
            candid::encode_args((
                KnownPrincipalType::CanisterIdSnsWasm,
                Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap(),
            ))
            .unwrap(),
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

    for i in 0..150 {
        pocket_ic.tick();
    }

    let alice_principal = get_mock_user_alice_principal_id();
    let alice_canister_id: Principal = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id,
            alice_principal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();

    let alice_initial_cycle_balance = pocket_ic.cycle_balance(alice_canister_id);

    let sns_wasm_w_canister_id = Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap();

    setup_sns_w_canister_for_creator_dao(&pocket_ic, super_admin);

    let deployed_cdao_canisters = setup_default_sns_creator_token(&pocket_ic, super_admin, alice_principal, alice_canister_id);

    let before_deleting_subnet_backup_capacity = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            super_admin,
            "get_subnet_backup_capacity",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let subnet_capacity: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_subnet_backup_capacity failed\n"),
            };
            subnet_capacity
        })
        .unwrap();

    pocket_ic
        .update_call(
            platform_canister_id,
            charlie_global_admin,
            "delete_all_sns_creator_token_in_the_network",
            candid::encode_one(alice_canister_id).unwrap(),
        )
        .unwrap();

    for _ in 0..50 {
        pocket_ic.tick();
    }

    let deployed_cdao_canisters = pocket_ic
        .query_call(
            alice_canister_id,
            alice_principal,
            "deployed_cdao_canisters",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let response: Vec<DeployedCdaoCanisters> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get deployed_cdao_canisters failed\n"),
            };
            response
        })
        .unwrap();

    assert_eq!(deployed_cdao_canisters.len(), 0);

    let after_deleting_backup_capacity = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            super_admin,
            "get_subnet_backup_capacity",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let subnet_capacity: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_subnet_backup_capacity failed\n"),
            };
            subnet_capacity
        })
        .unwrap();

    assert_eq!(
        before_deleting_subnet_backup_capacity + 5,
        after_deleting_backup_capacity
    );

}
