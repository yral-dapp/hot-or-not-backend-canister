use std::time::{Duration, SystemTime};

use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::common::types::known_principal::KnownPrincipalType;
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::get_mock_user_alice_principal_id,
};

#[test]
fn update_last_access_time_test() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();
    let user_index_canister_id: Principal = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .copied()
        .unwrap();
    let alice_principal_id: Principal = get_mock_user_alice_principal_id();

    let alice_canister_id: Principal = pocket_ic
        .update_call(
            user_index_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let result: Result<Principal, String> = candid::decode_one(&payload).unwrap();
                    result.unwrap()
                }
                _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"),
            }
        })
        .expect("Failed to call user_index_canister");

    let initial_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_438_993))
        .unwrap();
    pocket_ic.set_time(initial_time);

    let initial_last_access_time: SystemTime = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_last_access_time",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_access_time failed\n"),
            }
        })
        .expect("Failed to query last access time");

    assert_eq!(initial_last_access_time, initial_time);

    let updated_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_439_993))
        .unwrap();
    pocket_ic.set_time(updated_time);

    let update_response: Result<(), String> = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "update_last_access_time",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_last_access_time failed\n"),
            }
        })
        .expect("Failed to update last access time");

    assert!(update_response.is_ok());

    let updated_last_access_time: SystemTime = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_last_access_time",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_access_time failed\n"),
            }
        })
        .expect("Failed to query last access time");

    assert_eq!(updated_last_access_time, updated_time);
}