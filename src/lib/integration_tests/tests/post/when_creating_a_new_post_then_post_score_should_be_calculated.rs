use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::post::{
        PostDetailsForFrontend, PostDetailsFromFrontend,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::get_mock_user_alice_principal_id,
};

#[ignore]
#[test]
fn when_creating_a_new_post_then_post_score_should_be_calculated() {
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

    let newly_created_post_id: u64 = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "add_post_v2",
            candid::encode_args((PostDetailsFromFrontend {
                is_nsfw: false,
                description: "This is a fun video to watch".to_string(),
                hashtags: vec!["fun".to_string(), "video".to_string()],
                video_uid: "abcd#1234".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let result: Result<u64, String> = candid::decode_one(&payload).unwrap();
                    result.unwrap()
                }
                _ => panic!("\n🛑 add_post_v2 failed\n"),
            }
        })
        .expect("Failed to add post");

    let post_score: u64 = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_individual_post_details_by_id",
            candid::encode_args((newly_created_post_id,)).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let post_details: PostDetailsForFrontend = candid::decode_one(&payload).unwrap();
                    post_details.home_feed_ranking_score
                }
                _ => panic!("\n🛑 get_individual_post_details_by_id failed\n"),
            }
        })
        .expect("Failed to query post details");

    assert!(post_score > 0);
}