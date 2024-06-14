use std::collections::BTreeMap;

use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::kv_storage::{
        NamespaceErrors, NamespaceForFrontend,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_charlie_principal_id},
};

#[test]
fn create_new_namespace() {
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

    for i in 0..50 {
        pocket_ic.tick();
    }

    let alice_principal_id = get_mock_user_alice_principal_id();

    let alice_canister_id = pocket_ic.update_call(subnet_orchestrator_canister_id, alice_principal_id, "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer", candid::encode_one(()).unwrap())
    .map(|res| {
        let canister_id: Principal = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("Canister call failed")
        };
        canister_id
    })
    .unwrap();

    let app_proxy_canister = pocket_ic.create_canister();

    let namespace = pocket_ic
        .update_call(
            alice_canister_id,
            app_proxy_canister,
            "create_a_namespace",
            candid::encode_one("third_party_app".to_string()).unwrap(),
        )
        .map(|res| {
            let canister_id: Result<NamespaceForFrontend, NamespaceErrors> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id
        })
        .unwrap()
        .unwrap();

    let mut key_values: BTreeMap<String, String> = BTreeMap::new();
    key_values.insert("username".into(), "alice".into());
    key_values.insert("token_balance".into(), "1.00".into());
    pocket_ic
        .update_call(
            alice_canister_id,
            app_proxy_canister,
            "write_multiple_key_value_pairs",
            candid::encode_args((namespace.id, key_values)).unwrap(),
        )
        .map(|res| {
            let res: Result<(), NamespaceErrors> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            res
        })
        .unwrap()
        .unwrap();

    let prev_value = pocket_ic
        .update_call(
            alice_canister_id,
            app_proxy_canister,
            "write_key_value_pair",
            candid::encode_args((namespace.id, "token_balance".to_owned(), "10.00".to_owned()))
                .unwrap(),
        )
        .map(|res| {
            let res: Result<Option<String>, NamespaceErrors> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            res
        })
        .unwrap()
        .unwrap();

    assert_eq!(prev_value, Some("1.00".into()));

    //profile owners can delete value
    let deleted_value = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "delete_key_value_pair",
            candid::encode_args((namespace.id, "token_balance".to_owned())).unwrap(),
        )
        .map(|res| {
            let res: Result<Option<String>, NamespaceErrors> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            res
        })
        .unwrap()
        .unwrap();

    assert_eq!(deleted_value, Some("10.00".into()))
}
