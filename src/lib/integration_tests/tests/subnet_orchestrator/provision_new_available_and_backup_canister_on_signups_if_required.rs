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
    env::pocket_ic_env::get_new_pocket_ic_env, test_constants::get_mock_user_charlie_principal_id,
};

#[test]
pub fn provision_new_available_and_backup_canisters_on_signup_if_required() {
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

    for i in 0..130 {
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
            let available_capacity: u64 = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet available capacity call failed"),
            };
            available_capacity
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

    assert!(subnet_available_canister_cnt > 0);
}
