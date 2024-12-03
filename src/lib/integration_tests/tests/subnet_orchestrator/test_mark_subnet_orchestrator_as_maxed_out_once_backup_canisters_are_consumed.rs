use candid::{encode_one, Principal};
use pocket_ic::WasmResult;
use shared_utils::{
    common::types::known_principal::KnownPrincipalType,
    constant::{
        TEST_BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE,
        TEST_INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE,
    },
};
use test_utils::setup::{
    env::pocket_ic_env::{get_new_pocket_ic_env, provision_subnet_orchestrator_canister}, test_constants::get_mock_user_charlie_principal_id,
};

#[test]
fn test_mark_subnet_orchestrator_as_maxed_out_once_backup_canisters_are_consumed() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let charlie_global_admin = get_mock_user_charlie_principal_id();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "add_principal_as_global_admin",
            candid::encode_one(charlie_global_admin).unwrap(),
        )
        .unwrap();

    let subnet_orchestrator_canister_id = provision_subnet_orchestrator_canister(
        &pocket_ic,
        &known_principal,
        1,
        Some(charlie_global_admin),
    );

    for _ in 0..150 {
        pocket_ic.tick();
    }

    let subnet_available_canister_cnt = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_subnet_available_capacity",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let available_capacity: u64 = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet available capacity call failed"),
            };
            available_capacity
        })
        .unwrap();

    assert_eq!(
        subnet_available_canister_cnt,
        TEST_INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE
    );

    for _ in 0..10 {
        pocket_ic.tick();
    }
    let subnet_backup_canister_cnt = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_subnet_backup_capacity",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let backup_capacity: u64 = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet backup capacity call failed"),
            };
            backup_capacity
        })
        .unwrap();

    assert_eq!(
        subnet_backup_canister_cnt,
        TEST_BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE
    );

    /**************************** Provisioning individual canisters consuming all canisters ***************************************/

    for i in 0..TEST_INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE {
        let _individual_canister_id = pocket_ic
            .update_call(
                subnet_orchestrator_canister_id,
                Principal::self_authenticating((i + 1).to_ne_bytes()),
                "get_requester_principals_canister_id_create_if_not_exists",
                encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let result: Result<Principal, String> = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!(
                        "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                    ),
                };
                result
            })
            .unwrap()
            .unwrap();
    }

    let subnet_available_canister_cnt = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_subnet_available_capacity",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let available_capacity: u64 = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet available capacity call failed"),
            };
            available_capacity
        })
        .unwrap();

    /*****************************************************************************/

    for _ in 0..10 {
        pocket_ic.tick();
    }

    let subnet_available_canister_cnt = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_subnet_available_capacity",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let available_capacity: u64 = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet available capacity call failed"),
            };
            available_capacity
        })
        .unwrap();

    assert_eq!(
        subnet_available_canister_cnt,
        TEST_BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE
    );

    for i in TEST_INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE
        ..(TEST_INDIVIDUAL_USER_CANISTER_SUBNET_BATCH_SIZE + 10)
    {
        let _individual_canister_id = pocket_ic
            .update_call(
                subnet_orchestrator_canister_id,
                Principal::self_authenticating((i + 1).to_ne_bytes()),
                "get_requester_principals_canister_id_create_if_not_exists",
                encode_one(()).unwrap(),
            )
            .map(|reply_payload| {
                let result: Result<Principal, String> = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!(
                        "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                    ),
                };
                result
            })
            .unwrap()
            .unwrap();
    }

    for _ in 0..5 {
        pocket_ic.tick();
    }

    let subnet_orchestrator_with_capacity_left = pocket_ic
        .query_call(
            platform_canister_id,
            Principal::anonymous(),
            "get_all_available_subnet_orchestrators",
            candid::encode_one(()).unwrap(),
        )
        .map(|wasm_result| {
            let result: Vec<Principal> = match wasm_result {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(err) => panic!(
                    "\n call to get all available subnet orchestrators failed {} \n",
                    err
                ),
            };
            result
        })
        .unwrap();

    assert!(!subnet_orchestrator_with_capacity_left.contains(&subnet_orchestrator_canister_id));

    let subnet_available_canister_cnt = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_subnet_available_capacity",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let available_capacity: u64 = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet available capacity call failed"),
            };
            available_capacity
        })
        .unwrap();

    assert_eq!(subnet_available_canister_cnt, 0);

    let subnet_backup_canister_cnt = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_subnet_backup_capacity",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let backup_capacity: u64 = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet backup capacity call failed"),
            };
            backup_capacity
        })
        .unwrap();

    assert_eq!(subnet_backup_canister_cnt, 0);
}
