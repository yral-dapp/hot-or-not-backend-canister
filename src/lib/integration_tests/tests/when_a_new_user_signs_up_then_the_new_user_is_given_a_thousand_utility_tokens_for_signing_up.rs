use candid::Principal;
use ic_state_machine_tests::{CanisterId, StateMachine, WasmResult};

use shared_utils::{
    common::types::{
        known_principal::KnownPrincipalType,
        utility_token::token_event::{MintEvent, TokenEvent},
    },
    types::canister_specific::individual_user_template::error_types::GetUserUtilityTokenTransactionHistoryError,
};
use test_utils::setup::{
    env::v0::{
        get_canister_id_of_specific_type_from_principal_id_map,
        get_initialized_env_with_provisioned_known_canisters,
    },
    test_constants::get_alice_principal_id,
};

#[test]
fn when_a_new_user_signs_up_then_the_new_user_is_given_a_thousand_utility_tokens_for_signing_up() {
    // * Arrange
    let state_machine = StateMachine::new();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = get_canister_id_of_specific_type_from_principal_id_map(
        &known_principal_map,
        KnownPrincipalType::CanisterIdUserIndex,
    );
    let alice_principal_id = get_alice_principal_id();

    // * Act
    let alice_canister_id = state_machine.execute_ingress_as(
        alice_principal_id,
        user_index_canister_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let (alice_canister_id,): (Principal,) = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        alice_canister_id
    }).unwrap();

    let alice_utility_token_balance_after_signup = state_machine
        .query(
            CanisterId::new(alice_canister_id.into()).unwrap(),
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

    println!(
        "🧪 alice_utility_token_balance_after_signup: {}",
        alice_utility_token_balance_after_signup
    );

    // * getting canister id again to check if token value increased
    state_machine.execute_ingress_as(
        alice_principal_id,
        user_index_canister_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let (alice_canister_id,): (Principal,) = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        alice_canister_id
    }).unwrap();

    let alice_utility_token_balance_after_calling_get_canister_id_again = state_machine
        .query(
            CanisterId::new(alice_canister_id.into()).unwrap(),
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

    println!(
        "🧪 alice_utility_token_balance_after_calling_get_canister_id_again: {}",
        alice_utility_token_balance_after_calling_get_canister_id_again
    );

    let alice_utility_token_transaction_history_after_signup: Vec<(u64, TokenEvent)> =
        state_machine
            .query(
                CanisterId::new(alice_canister_id.into()).unwrap(),
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

    println!(
        "🧪 alice_utility_token_transaction_history_after_signup: {:#?}",
        alice_utility_token_transaction_history_after_signup
    );

    // * Assert
    assert_eq!(alice_utility_token_balance_after_signup, 1000);
    assert_eq!(
        alice_utility_token_balance_after_calling_get_canister_id_again,
        1000
    );
    assert_eq!(
        alice_utility_token_balance_after_signup,
        alice_utility_token_balance_after_calling_get_canister_id_again
    );
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
            new_user_principal_id: alice_principal_id.0
        },
    );
}
