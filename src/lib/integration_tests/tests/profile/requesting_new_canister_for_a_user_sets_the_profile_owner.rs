use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontend,
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::get_mock_user_alice_principal_id,
};

#[test]
fn requesting_new_canister_for_a_user_sets_the_profile_owner() {
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

    let profile_details: UserProfileDetailsForFrontend = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_profile_details",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile_details failed\n"),
            }
        })
        .expect("Failed to query profile details");

    assert_eq!(profile_details.principal_id, alice_principal_id);
}