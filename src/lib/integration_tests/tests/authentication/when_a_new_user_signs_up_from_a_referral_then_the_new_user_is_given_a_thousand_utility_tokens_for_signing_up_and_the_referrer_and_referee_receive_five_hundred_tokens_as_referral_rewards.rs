use candid::Principal;
use ic_test_state_machine_client::WasmResult;
use shared_utils::{
    common::types::{
        known_principal::KnownPrincipalType,
        utility_token::token_event::{MintEvent, TokenEvent},
    },
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
    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();

    let alice_canister_id = state_machine.update_call(
        *user_index_canister_id,
        alice_principal_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let alice_canister_id: Principal = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        alice_canister_id
    }).unwrap();

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

    // * getting canister id again to check if token value increased
    state_machine.update_call(
        *user_index_canister_id,
        alice_principal_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let alice_canister_id: Principal = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        alice_canister_id
    }).unwrap();

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
                candid::encode_args((0 as u64, 10 as u64)).unwrap(),
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

    let bob_canister_id = state_machine.update_call(
        *user_index_canister_id,
        bob_principal_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(Some(alice_principal_id)).unwrap(),
    ).map(|reply_payload| {
        let bob_canister_id: Principal = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        bob_canister_id
    }).unwrap();

    let alice_utility_token_balance_after_referral = state_machine
        .query_call(
            alice_canister_id.into(),
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
                candid::encode_args((0 as u64, 10 as u64)).unwrap(),
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
                candid::encode_args((0 as u64, 10 as u64)).unwrap(),
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
