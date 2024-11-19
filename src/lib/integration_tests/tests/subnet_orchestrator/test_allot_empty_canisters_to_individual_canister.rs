use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::common::types::known_principal::KnownPrincipalType;
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_charlie_principal_id},
};

#[test]
fn test_allot_empty_canisters_to_individual_canister() {
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

    for i in 0..120 {
        pocket_ic.tick();
    }

    let alice_yral_principal_id = get_mock_user_alice_principal_id();
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

    let mut intial_subnet_backup_capacity = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            super_admin,
            "get_subnet_backup_capacity",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let subnet_capacity: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_subnet_backup_capacity failed\n"),
            };
            subnet_capacity
        })
        .unwrap();

    for _ in 0..10 {
        let alloted_canister = pocket_ic
            .update_call(
                subnet_orchestrator_canister_id,
                alice_yral_canister_id,
                "allot_empty_canister",
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

        intial_subnet_backup_capacity -= 1;

        let alloted_canister_status = pocket_ic
            .canister_status(alloted_canister, Some(alice_yral_canister_id))
            .unwrap();

        assert!(alloted_canister_status
            .settings
            .controllers
            .contains(&alice_yral_canister_id));
    }
}
