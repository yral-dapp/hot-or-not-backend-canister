use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::common::types::known_principal::KnownPrincipalType;
use test_utils::setup::{
    env::pocket_ic_env::{get_new_pocket_ic_env, provision_n_subnet_orchestrator_canisters},
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_bob_principal_id},
};

#[test]
fn recharge_individual_canister_when_requested() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();

    let alice_principal = get_mock_user_alice_principal_id();
    let bob_princpal = get_mock_user_bob_principal_id();
    let mut bob_winnigs = 0_u64;
    let mut charlie_winnings = 0_u64;
    let mut dan_winnings = 0_u64;
    let mut lucy_winnings = 0_u64;
    let mut tom_winnigns = 0_u64;

    let subnet_orchestrators = provision_n_subnet_orchestrator_canisters(
        &pocket_ic,
        &known_principal_map,
        2,
        None,
    );
    let subnet_orchestrator_canister_id_0 = subnet_orchestrators[0];

    let subnet_orchestrator_canister_id_1 = subnet_orchestrators[1];
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
