use candid::Principal;
use ic_test_state_machine_client::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::session::SessionType,
    common::types::known_principal::KnownPrincipalType, constant::GLOBAL_SUPER_ADMIN_USER_ID,
};
use test_utils::setup::{
    env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
    test_constants::get_mock_user_alice_principal_id,
};

#[test]
fn update_session_type_tests() {
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

    let session_type = state_machine
        .query_call(
            alice_canister_id,
            alice_principal_id,
            "get_session_type",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let session_type_res: Result<SessionType, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_session_type failed\n"),
            };
            session_type_res
        })
        .unwrap()
        .unwrap();
    assert_eq!(session_type, SessionType::AnonymousSession);

    // user cannot update the session type
    state_machine
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "update_session_type",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .unwrap();

    let session_type = state_machine
        .query_call(
            alice_canister_id,
            alice_principal_id,
            "get_session_type",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let session_type_res: Result<SessionType, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_session_type failed\n"),
            };
            session_type_res
        })
        .unwrap()
        .unwrap();

    assert_eq!(session_type, SessionType::AnonymousSession);

    // global admin can update the session type of canister
    state_machine
        .update_call(
            alice_canister_id,
            Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap(),
            "update_session_type",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .unwrap();

    let session_type = state_machine
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_session_type",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let session_type_res: Result<SessionType, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_session_type failed\n"),
            };
            session_type_res
        })
        .unwrap()
        .unwrap();

    assert_eq!(session_type, SessionType::RegisteredSession);

    //once set to registered session cannot update back to Anonymous Session
    state_machine
        .update_call(
            alice_canister_id,
            Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap(),
            "update_session_type",
            candid::encode_one(SessionType::AnonymousSession).unwrap(),
        )
        .unwrap();

    let session_type = state_machine
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_session_type",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let session_type_res: Result<SessionType, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_session_type failed\n"),
            };
            session_type_res
        })
        .unwrap()
        .unwrap();

    assert_eq!(session_type, SessionType::RegisteredSession)
}
