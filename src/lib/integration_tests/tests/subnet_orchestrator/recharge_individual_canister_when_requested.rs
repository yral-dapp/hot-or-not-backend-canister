use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::common::types::known_principal::KnownPrincipalType;
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_bob_principal_id},
};

#[test]
fn recharge_individual_canister_when_requested() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();

    let alice_principal = get_mock_user_alice_principal_id();
    let bob_princpal = get_mock_user_bob_principal_id();

    let platform_orchestrator_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .copied()
        .unwrap();

    let global_admin_principal = known_principal_map
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .copied()
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let subnet_orchestrator_canister_id_0 = pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            global_admin_principal,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[0]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    let subnet_orchestrator_canister_id_1 = pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            global_admin_principal,
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

    for _ in 0..50 {
        pocket_ic.tick();
    }

    //Post Creator Canister
    let alice_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_0,
            alice_principal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canister_id_res: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            canister_id_res
        })
        .unwrap()
        .unwrap();

    let bob_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_1,
            bob_princpal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canister_id_res: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            canister_id_res
        })
        .unwrap()
        .unwrap();

    let deposit_cycle_call_to_subnet_orchestrator_from_bob = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_0,
            bob_canister_id,
            "recharge_individual_user_canister",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let res: Result<(), String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("recharge_individual_user_canister call failed"),
            };
            res
        })
        .unwrap();

    assert!(deposit_cycle_call_to_subnet_orchestrator_from_bob.is_err());

    let deposit_cycle_call_to_subnet_orchestrator_from_alice = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id_0,
            alice_canister_id,
            "recharge_individual_user_canister",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let res: Result<(), String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("recharge_individual_user_canister call failed"),
            };
            res
        })
        .unwrap();

    assert!(deposit_cycle_call_to_subnet_orchestrator_from_alice.is_ok());
}
