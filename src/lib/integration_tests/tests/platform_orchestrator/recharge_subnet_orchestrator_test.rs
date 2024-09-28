use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    common::types::known_principal::KnownPrincipalType,
    constant::{
        SUBNET_ORCHESTRATOR_CANISTER_CYCLES_THRESHOLD, SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES,
    },
};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env;

#[test]
pub fn recharge_subnet_orchestrator_test() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();

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

    pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            subnet_orchestrator_canister_id_0,
            "recharge_subnet_orchestrator",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let res: Result<(), String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("recharge subnet orchestrator call failed"),
            };
            res
        })
        .unwrap()
        .unwrap();

    let subnet_orchestrator_cycle_balance =
        pocket_ic.cycle_balance(subnet_orchestrator_canister_id_0);

    assert!(subnet_orchestrator_cycle_balance > SUBNET_ORCHESTRATOR_CANISTER_CYCLES_THRESHOLD);
    assert!(subnet_orchestrator_cycle_balance < SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES);
}
