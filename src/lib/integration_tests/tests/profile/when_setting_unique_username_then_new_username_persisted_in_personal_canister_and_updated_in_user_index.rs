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
fn when_setting_unique_username_then_new_username_persisted_in_personal_canister_and_updated_in_user_index() {
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

    let profile_details_from_user_canister: UserProfileDetailsForFrontend = pocket_ic
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

    assert!(profile_details_from_user_canister.unique_user_name.is_none());

    let update_profile_response: Result<(), String> = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "update_profile_set_unique_username_once",
            candid::encode_one("cool_alice_1234".to_string()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_profile_set_unique_username_once failed\n"),
            }
        })
        .expect("Failed to update profile");

    assert!(update_profile_response.is_ok());

    let profile_details_from_user_canister: UserProfileDetailsForFrontend = pocket_ic
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

    assert_eq!(
        profile_details_from_user_canister.unique_user_name,
        Some("cool_alice_1234".to_string())
    );

    let is_alice_username_taken: bool = pocket_ic
        .query_call(
            user_index_canister_id,
            Principal::anonymous(),
            "is_user_name_taken",
            candid::encode_one("cool_alice_1234".to_string()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 is_user_name_taken failed\n"),
            }
        })
        .expect("Failed to query username status");

    assert!(is_alice_username_taken);

    println!(
        "🧪 profile_details_from_user_canister: {:?}",
        profile_details_from_user_canister
    );

    println!("🧪 is_alice_username_taken: {:?}", is_alice_username_taken);

    let alice_canister_id_corresponding_to_username: Option<Principal> = pocket_ic
        .query_call(
            user_index_canister_id,
            Principal::anonymous(),
            "get_user_canister_id_from_unique_user_name",
            candid::encode_one("cool_alice_1234".to_string()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_canister_id_from_unique_user_name failed\n"),
            }
        })
        .expect("Failed to query canister id by username");

    println!(
        "🧪 alice_canister_id_corresponding_to_username: {:?}",
        alice_canister_id_corresponding_to_username
    );

    let alice_canister_id_corresponding_to_principal_id: Option<Principal> = pocket_ic
        .query_call(
            user_index_canister_id,
            Principal::anonymous(),
            "get_user_canister_id_from_user_principal_id",
            candid::encode_one(alice_principal_id).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_canister_id_from_user_principal_id failed\n"),
            }
        })
        .expect("Failed to query canister id by principal");

    assert_eq!(
        alice_canister_id_corresponding_to_username,
        alice_canister_id_corresponding_to_principal_id
    );

    println!(
        "🧪 alice_canister_id_corresponding_to_principal_id: {:?}",
        alice_canister_id_corresponding_to_principal_id
    );
}