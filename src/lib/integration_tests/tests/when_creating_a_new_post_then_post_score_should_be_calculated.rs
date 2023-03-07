use candid::Principal;
use ic_state_machine_tests::{CanisterId, PrincipalId, StateMachine, WasmResult};
use shared_utils::{
    canister_specific::individual_user_template::types::post::{
        PostDetailsForFrontend, PostDetailsFromFrontend,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env_v0::{
        get_canister_id_of_specific_type_from_principal_id_map,
        get_initialized_env_with_provisioned_known_canisters,
    },
    test_constants::get_alice_principal_id,
};

#[test]
fn when_creating_a_new_post_then_post_score_should_be_calculated() {
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

    let newly_created_post_id = state_machine
        .execute_ingress_as(
            alice_principal_id,
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "add_post",
            candid::encode_args((PostDetailsFromFrontend {
                description: "This is a fun video to watch".to_string(),
                hashtags: vec!["fun".to_string(), "video".to_string()],
                video_uid: "abcd#1234".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .map(|reply_payload| {
            let (newly_created_post_id,): (u64,) = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id
        })
        .unwrap();

    let post_score = state_machine
        .query(
            CanisterId::new(PrincipalId(alice_canister_id)).unwrap(),
            "get_individual_post_details_by_id",
            candid::encode_args((newly_created_post_id,)).unwrap(),
        )
        .map(|reply_payload| {
            let (post_details,): (PostDetailsForFrontend,) = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_args(&payload).unwrap(),
                _ => panic!("\n🛑 get_individual_post_details_by_id failed\n"),
            };
            post_details.home_feed_ranking_score
        })
        .unwrap();

    // * Assert
    assert!(post_score > 0);
}
