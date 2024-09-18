use candid::Principal;
use ic_test_state_machine_client::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        profile::{UserCanisterDetails, UserProfileDetailsForFrontend},
        session::SessionType,
    },
    common::types::{
        known_principal::KnownPrincipalType,
        utility_token::token_event::{MintEvent, TokenEvent},
    },
    constant::GLOBAL_SUPER_ADMIN_USER_ID,
    types::canister_specific::individual_user_template::error_types::GetUserUtilityTokenTransactionHistoryError,
};
use test_utils::setup::{
    env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_bob_principal_id},
};

#[test]
fn when_a_new_user_signs_up_from_a_referral_then_the_new_user_is_given_a_thousand_utility_tokens_for_signing_up_and_the_referrer_and_referee_receive_five_hundred_tokens_as_referral_rewards(
) {
    let state_machine = get_new_state_machine();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();
    let global_admin_principal = Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap();
    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();

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

    let alice_utility_token_balance_after_signup = state_machine
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            balance
        })
        .unwrap();

    assert_eq!(alice_utility_token_balance_after_signup, 1000);

    // check version of alice canister
    let alice_canister_version = state_machine
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_version",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let version: String = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            version
        })
        .unwrap();

    assert!(alice_canister_version.eq("v1.0.0"));

    // * getting canister id again to check if token value increased
    state_machine
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

    let alice_utility_token_balance_after_calling_get_canister_id_again = state_machine
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            balance
        })
        .unwrap();

    assert_eq!(
        alice_utility_token_balance_after_calling_get_canister_id_again,
        1000
    );

    assert_eq!(
        alice_utility_token_balance_after_signup,
        alice_utility_token_balance_after_calling_get_canister_id_again
    );

    println!(
        "🧪 alice_utility_token_balance_after_calling_get_canister_id_again: {}",
        alice_utility_token_balance_after_calling_get_canister_id_again
    );

    let alice_utility_token_transaction_history_after_signup: Vec<(u64, TokenEvent)> =
        state_machine
            .query_call(
                alice_canister_id,
                Principal::anonymous(),
                "get_user_utility_token_transaction_history_with_pagination",
                candid::encode_args((0_u64, 10_u64)).unwrap(),
            )
            .map(|reply_payload| {
                let response: Result<
                    Vec<(u64, TokenEvent)>,
                    GetUserUtilityTokenTransactionHistoryError,
                > = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!(
                        "\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"
                    ),
                };
                response
            })
            .unwrap()
            .unwrap();

    assert_eq!(
        alice_utility_token_transaction_history_after_signup.len(),
        1
    );

    assert_eq!(
        match alice_utility_token_transaction_history_after_signup[0]
            .1
            .clone()
        {
            TokenEvent::Mint { details, .. } => details,
            _ => {
                MintEvent::NewUserSignup {
                    new_user_principal_id: Principal::anonymous(),
                }
            }
        },
        MintEvent::NewUserSignup {
            new_user_principal_id: alice_principal_id
        },
    );

    println!(
        "🧪 alice_utility_token_transaction_history_after_signup: {:#?}",
        alice_utility_token_transaction_history_after_signup
    );

    state_machine
        .update_call(
            alice_canister_id,
            global_admin_principal,
            "update_session_type",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .unwrap();

    let bob_canister_id = state_machine
        .update_call(
            *user_index_canister_id,
            bob_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let bob_canister_id: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            bob_canister_id
        })
        .unwrap()
        .unwrap();

    state_machine
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
        .unwrap();

    let bob_profile_details = state_machine
        .query_call(
            bob_canister_id,
            Principal::anonymous(),
            "get_profile_details",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile_details_from_user_canister: UserProfileDetailsForFrontend =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile_details failed\n"),
                };
            profile_details_from_user_canister
        })
        .unwrap();

    assert!(bob_profile_details.referrer_details.is_some());
    let bob_referrer_details = bob_profile_details.referrer_details.unwrap();
    assert_eq!(bob_referrer_details.profile_owner, alice_principal_id);
    assert_eq!(bob_referrer_details.user_canister_id, alice_canister_id);

    state_machine
        .update_call(
            bob_canister_id,
            global_admin_principal,
            "update_session_type",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .unwrap();

    state_machine
        .update_call(
            *user_index_canister_id,
            Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap(),
            "issue_rewards_for_referral",
            candid::encode_args((alice_canister_id, alice_principal_id, bob_principal_id)).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(err) => panic!("{err}"),
            };
            result
        })
        .unwrap()
        .unwrap();

    state_machine
        .update_call(
            *user_index_canister_id,
            Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap(),
            "issue_rewards_for_referral",
            candid::encode_args((bob_canister_id, alice_principal_id, bob_principal_id)).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(err) => panic!("{err}"),
            };
            result
        })
        .unwrap()
        .unwrap();

    let alice_utility_token_balance_after_referral = state_machine
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            balance
        })
        .unwrap();

    assert_eq!(alice_utility_token_balance_after_referral, 1500);

    let bob_utility_token_balance_after_referral = state_machine
        .query_call(
            bob_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_utility_token_balance failed\n"),
            };
            balance
        })
        .unwrap();

    assert_eq!(bob_utility_token_balance_after_referral, 1500);

    let alice_utility_token_transaction_history_after_referral: Vec<(u64, TokenEvent)> =
        state_machine
            .query_call(
                alice_canister_id,
                Principal::anonymous(),
                "get_user_utility_token_transaction_history_with_pagination",
                candid::encode_args((0_u64, 10_u64)).unwrap(),
            )
            .map(|reply_payload| {
                let response: Result<
                    Vec<(u64, TokenEvent)>,
                    GetUserUtilityTokenTransactionHistoryError,
                > = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!(
                        "\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"
                    ),
                };
                response
            })
            .unwrap()
            .unwrap();

    assert_eq!(
        alice_utility_token_transaction_history_after_referral.len(),
        2
    );

    assert_eq!(
        match alice_utility_token_transaction_history_after_referral[0]
            .1
            .clone()
        {
            TokenEvent::Mint { details, .. } => details,
            _ => {
                MintEvent::Referral {
                    referee_user_principal_id: Principal::anonymous(),
                    referrer_user_principal_id: Principal::anonymous(),
                }
            }
        },
        MintEvent::Referral {
            referrer_user_principal_id: alice_principal_id,
            referee_user_principal_id: bob_principal_id,
        },
    );

    assert_eq!(
        match alice_utility_token_transaction_history_after_referral[1]
            .1
            .clone()
        {
            TokenEvent::Mint { details, .. } => details,
            _ => {
                MintEvent::NewUserSignup {
                    new_user_principal_id: Principal::anonymous(),
                }
            }
        },
        MintEvent::NewUserSignup {
            new_user_principal_id: alice_principal_id
        },
    );

    println!(
        "🧪 alice_utility_token_transaction_history_after_referral: {:#?}",
        alice_utility_token_transaction_history_after_referral
    );

    let bob_utility_token_transaction_history_after_referral: Vec<(u64, TokenEvent)> =
        state_machine
            .query_call(
                bob_canister_id,
                Principal::anonymous(),
                "get_user_utility_token_transaction_history_with_pagination",
                candid::encode_args((0_u64, 10_u64)).unwrap(),
            )
            .map(|reply_payload| {
                let response: Result<
                    Vec<(u64, TokenEvent)>,
                    GetUserUtilityTokenTransactionHistoryError,
                > = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!(
                        "\n🛑 get_user_utility_token_transaction_history_with_pagination failed\n"
                    ),
                };
                response
            })
            .unwrap()
            .unwrap();

    assert_eq!(
        bob_utility_token_transaction_history_after_referral.len(),
        2
    );

    assert_eq!(
        match bob_utility_token_transaction_history_after_referral[0]
            .1
            .clone()
        {
            TokenEvent::Mint { details, .. } => details,
            _ => {
                MintEvent::Referral {
                    referee_user_principal_id: Principal::anonymous(),
                    referrer_user_principal_id: Principal::anonymous(),
                }
            }
        },
        MintEvent::Referral {
            referrer_user_principal_id: alice_principal_id,
            referee_user_principal_id: bob_principal_id,
        },
    );

    assert_eq!(
        match bob_utility_token_transaction_history_after_referral[1]
            .1
            .clone()
        {
            TokenEvent::Mint { details, .. } => details,
            _ => {
                MintEvent::NewUserSignup {
                    new_user_principal_id: Principal::anonymous(),
                }
            }
        },
        MintEvent::NewUserSignup {
            new_user_principal_id: bob_principal_id
        },
    );

    println!(
        "🧪 alice_utility_token_transaction_history_after_referral: {:#?}",
        alice_utility_token_transaction_history_after_referral
    );
}
