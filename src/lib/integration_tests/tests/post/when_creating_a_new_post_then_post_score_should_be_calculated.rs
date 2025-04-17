use std::time::{Duration, SystemTime};

use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::post::PostDetailsFromFrontend,
    common::types::{
        known_principal::KnownPrincipalType, top_posts::post_score_index_item::PostScoreIndexItem,
    },
    types::canister_specific::post_cache::error_types::TopPostsFetchError,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::get_mock_user_alice_principal_id,
};

#[test]
fn when_creating_a_new_post_then_post_score_should_be_calculated() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();
    let user_index_canister_id: Principal = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .copied()
        .unwrap();
    let post_cache_canister_id: Principal = known_principal_map
        .get(&KnownPrincipalType::CanisterIdPostCache)
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

    let post_creation_time = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(1_678_438_993))
        .unwrap();
    pocket_ic.set_time(post_creation_time);

    let newly_created_post_id: u64 = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "add_post_v2",
            candid::encode_one(PostDetailsFromFrontend {
                description: "This is a fun video to watch".to_string(),
                hashtags: vec!["fun".to_string(), "video".to_string()],
                video_uid: "abcd#1234".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
                is_nsfw: false,
            })
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

    let returned_posts: Vec<PostScoreIndexItem> = pocket_ic
        .query_call(
            post_cache_canister_id,
            Principal::anonymous(),
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed_cursor",
            candid::encode_args((0_u64, 10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> =
                        candid::decode_one(&payload).unwrap();
                    returned_posts.unwrap()
                }
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed_cursor failed\n"),
            }
        })
        .expect("Failed to query post cache");

    assert_eq!(returned_posts.len(), 1);
    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 3000);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);

    let returned_posts: Vec<PostScoreIndexItem> = pocket_ic
        .query_call(
            post_cache_canister_id,
            Principal::anonymous(),
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor",
            candid::encode_args((0_u64, 10_u64)).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => {
                    let returned_posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> =
                        candid::decode_one(&payload).unwrap();
                    returned_posts.unwrap()
                }
                _ => panic!("\n🛑 get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor failed\n"),
            }
        })
        .expect("Failed to query post cache");

    assert_eq!(returned_posts.len(), 1);
    let returned_post = returned_posts.get(0).unwrap();
    assert_eq!(returned_post.post_id, newly_created_post_id);
    assert_eq!(returned_post.score, 3000);
    assert_eq!(returned_post.publisher_canister_id, alice_canister_id);
}