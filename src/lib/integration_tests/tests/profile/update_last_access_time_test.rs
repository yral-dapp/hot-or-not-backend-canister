use candid::Principal;
use ic_test_state_machine_client::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        profile::UserProfileDetailsForFrontend, session::SessionType,
    },
    common::types::known_principal::KnownPrincipalType,
    constant::GLOBAL_SUPER_ADMIN_USER_ID,
};
use test_utils::setup::{
    env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
    test_constants::get_mock_user_alice_principal_id,
};

#[test]
fn update_last_access_time_test() {
    let state_machine = get_new_state_machine();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();
    let alice_principal_id = get_mock_user_alice_principal_id();

    let alice_canister_id = state_machine
        .update_call(
            *user_index_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let alice_canister_id: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            alice_canister_id
        })
        .unwrap()
        .unwrap();

    let update_last_access_time_result = state_machine
        .update_call(
            alice_canister_id,
            Principal::anonymous(),
            "update_last_access_time",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .map(|reply_payload| {
            let update_last_access_time_res: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_last_access_time failed\n"),
            };
            update_last_access_time_res
        })
        .unwrap();

    assert!(update_last_access_time_result.is_err());

    let update_last_access_time_result = state_machine
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "update_last_access_time",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .map(|reply_payload| {
            let update_last_access_time_res: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_last_access_time failed\n"),
            };
            update_last_access_time_res
        })
        .unwrap();

    update_last_access_time_result.unwrap();
}
