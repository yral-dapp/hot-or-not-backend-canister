use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        profile::{UserCanisterDetails, UserProfileDetailsForFrontend},
        session::SessionType,
    },
    common::types::{
        known_principal::KnownPrincipalType,
        utility_token::token_event::{MintEvent, TokenEvent},
    },
    types::canister_specific::individual_user_template::error_types::GetUserUtilityTokenTransactionHistoryError,
};
use test_utils::setup::{
    env::{pocket_ic_env::get_new_pocket_ic_env, pocket_ic_init::get_initialized_env_with_provisioned_known_canisters},
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_bob_principal_id},
};

#[test]
fn when_a_new_user_signs_up_from_a_referral_then_the_new_user_is_given_a_thousand_utility_tokens_for_signing_up_and_the_referrer_and_referee_receive_five_hundred_tokens_as_referral_rewards() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();
    let user_index_canister_id: Principal = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .copied()
        .unwrap();
    let global_admin_principal: Principal = known_principal_map
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .copied()
        .unwrap();
    let  alice_principal_id: Principal = get_mock_user_alice_principal_id();
    let bob_principal_id: Principal = get_mock_user_bob_principal_id();

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

    let alice_utility_token_balance_after_signup: u64 = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            }
        })
        .expect("Failed to query alice_canister");

    assert_eq!(alice_utility_token_balance_after_signup, 1000);

    let alice_canister_version: String = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_version",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_version failed\n"),
            }
        })
        .expect("Failed to query alice_canister");

    assert_eq!(alice_canister_version, "v1.0.0");

    let _alice_canister_id: Principal = pocket_ic
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

    let alice_utility_token_balance_after_calling_get_canister_id_again: u64 = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            }
        })
        .expect("Failed to query alice_canister");

    assert_eq!(alice_utility_token_balance_after_calling_get_canister_id_again, 1000);

    let alice_utility_token_transaction_history_after_signup: Vec<(u64, TokenEvent)> = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0_u64, 10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let response: Result<Vec<(u64, TokenEvent)>, GetUserUtilityTokenTransactionHistoryError> =
                        candid::decode_one(&payload).unwrap();
                    response.unwrap()
                }
                _ => panic!("\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"),
            }
        })
        .expect("Failed to query alice_canister");

    assert_eq!(alice_utility_token_transaction_history_after_signup.len(), 1);
    assert_eq!(
        match &alice_utility_token_transaction_history_after_signup[0].1 {
            TokenEvent::Mint { details, .. } => details,
            _ => panic!("Expected Mint event"),
        },
        &MintEvent::NewUserSignup {
            new_user_principal_id: alice_principal_id
        }
    );

    pocket_ic
        .update_call(
            alice_canister_id,
            global_admin_principal,
            "update_session_type",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .expect("Failed to update session type for Alice");

    let bob_canister_id: Principal = pocket_ic
        .update_call(
            user_index_canister_id,
            bob_principal_id,
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

    pocket_ic
        .update_call(
            bob_canister_id,
            bob_principal_id,
            "update_referrer_details",
            candid::encode_one(UserCanisterDetails {
                profile_owner: alice_principal_id,
                user_canister_id: alice_canister_id,
            })
            .unwrap(),
        )
        .expect("Failed to update referrer details for Bob");

    let bob_profile_details: UserProfileDetailsForFrontend = pocket_ic
        .query_call(
            bob_canister_id,
            Principal::anonymous(),
            "get_profile_details",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile_details failed\n"),
            }
        })
        .expect("Failed to query bob_canister");

    assert!(bob_profile_details.referrer_details.is_some());
    let bob_referrer_details: UserCanisterDetails = bob_profile_details.referrer_details.unwrap();
    assert_eq!(bob_referrer_details.profile_owner, alice_principal_id);
    assert_eq!(bob_referrer_details.user_canister_id, alice_canister_id);

    pocket_ic
        .update_call(
            bob_canister_id,
            global_admin_principal,
            "update_session_type",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .expect("Failed to update session type for Bob");

    pocket_ic
        .update_call(
            user_index_canister_id,
            global_admin_principal,
            "issue_rewards_for_referral",
            candid::encode_args((alice_canister_id, alice_principal_id, bob_principal_id)).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let result: Result<String, String> = candid::decode_one(&payload).unwrap();
                    result.unwrap()
                }
                e => panic!("\n🛑 issue_rewards_for_referral failed\n{e:?}"),
            }
        })
        .expect("Failed to issue referral rewards for Alice");

    pocket_ic
        .update_call(
            user_index_canister_id,
            global_admin_principal,
            "issue_rewards_for_referral",
            candid::encode_args((bob_canister_id, alice_principal_id, bob_principal_id)).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let result: Result<String, String> = candid::decode_one(&payload).unwrap();
                    result.unwrap()
                }
                _ => panic!("\n🛑 issue_rewards_for_referral failed\n"),
            }
        })
        .expect("Failed to issue referral rewards for Bob");

    let alice_utility_token_balance_after_referral: u64 = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            }
        })
        .expect("Failed to query alice_canister");

    assert_eq!(alice_utility_token_balance_after_referral, 1500);

    let bob_utility_token_balance_after_referral: u64 = pocket_ic
        .query_call(
            bob_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            }
        })
        .expect("Failed to query bob_canister");

    assert_eq!(bob_utility_token_balance_after_referral, 1500);

    let alice_utility_token_transaction_history_after_referral: Vec<(u64, TokenEvent)> = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0_u64, 10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let response: Result<Vec<(u64, TokenEvent)>, GetUserUtilityTokenTransactionHistoryError> =
                        candid::decode_one(&payload).unwrap();
                    response.unwrap()
                }
                _ => panic!("\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"),
            }
        })
        .expect("Failed to query alice_canister");

    assert_eq!(alice_utility_token_transaction_history_after_referral.len(), 2);
    assert_eq!(
        match &alice_utility_token_transaction_history_after_referral[0].1 {
            TokenEvent::Mint { details, .. } => details,
            _ => panic!("Expected Mint event"),
        },
        &MintEvent::Referral {
            referrer_user_principal_id: alice_principal_id,
            referee_user_principal_id: bob_principal_id,
        }
    );
    assert_eq!(
        match &alice_utility_token_transaction_history_after_referral[1].1 {
            TokenEvent::Mint { details, .. } => details,
            _ => panic!("Expected Mint event"),
        },
        &MintEvent::NewUserSignup {
            new_user_principal_id: alice_principal_id
        }
    );

    let bob_utility_token_transaction_history_after_referral: Vec<(u64, TokenEvent)> = pocket_ic
        .query_call(
            bob_canister_id,
            Principal::anonymous(),
            "get_user_utility_token_transaction_history_with_pagination",
            candid::encode_args((0_u64, 10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let response: Result<Vec<(u64, TokenEvent)>, GetUserUtilityTokenTransactionHistoryError> =
                        candid::decode_one(&payload).unwrap();
                    response.unwrap()
                }
                _ => panic!("\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"),
            }
        })
        .expect("Failed to query bob_canister");

    assert_eq!(bob_utility_token_transaction_history_after_referral.len(), 2);
    assert_eq!(
        match &bob_utility_token_transaction_history_after_referral[0].1 {
            TokenEvent::Mint { details, .. } => details,
            _ => panic!("Expected Mint event"),
        },
        &MintEvent::Referral {
            referrer_user_principal_id: alice_principal_id,
            referee_user_principal_id: bob_principal_id,
        }
    );
    assert_eq!(
        match &bob_utility_token_transaction_history_after_referral[1].1 {
            TokenEvent::Mint { details, .. } => details,
            _ => panic!("Expected Mint event"),
        },
        &MintEvent::NewUserSignup {
            new_user_principal_id: bob_principal_id
        }
    );
}