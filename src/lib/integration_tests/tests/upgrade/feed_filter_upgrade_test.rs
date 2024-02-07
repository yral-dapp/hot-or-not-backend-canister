use std::{collections::HashMap, thread, time::Duration};

use candid::{encode_one, CandidType, Deserialize};
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::IndividualUserTemplateInitArgs,
            post::{PostDetailsForFrontend, PostDetailsFromFrontend},
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::types::{
        known_principal::{KnownPrincipalMap, KnownPrincipalType},
        top_posts::post_score_index_item::{PostScoreIndexItem, PostScoreIndexItemV1},
    },
    types::canister_specific::post_cache::error_types::TopPostsFetchError,
};
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    get_mock_user_charlie_principal_id,
};

const OLD_POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/old_post_cache.wasm.gz";
const OLD_INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/old_individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";
const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";

#[derive(Deserialize, CandidType, Default)]
struct OldPostCacheInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMap>,
}

#[cfg(feature = "feed_filter_upgrade_test")]
#[test]
fn feed_filter_upgrade_test() {
    let pic = PocketIc::new();

    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let admin_principal_id = get_mock_user_charlie_principal_id();

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let post_cache_wasm_bytes = old_post_cache_canister_wasm();
    let post_cache_args = OldPostCacheInitArgs {
        known_principal_ids: None,
    };
    let post_cache_args_bytes = encode_one(post_cache_args).unwrap();
    pic.install_canister(
        post_cache_canister_id,
        post_cache_wasm_bytes,
        post_cache_args_bytes,
        None,
    );

    // Individual template canisters

    let individual_template_wasm_bytes = old_individual_template_canister_wasm();
    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );

    // Init individual template canister - alice

    let alice_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(alice_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(alice_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        alice_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init individual template canister - bob

    let bob_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(bob_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(bob_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        bob_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Create posts
    // Alice creates a post

    let alice_post_1 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(alice_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    let alice_post_2 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch 2".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#12345".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(alice_post_2).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // Bob creates a post

    let bob_post_1 = PostDetailsFromFrontend {
        is_nsfw: true,
        description: "This is a fun video to watch - bob".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234bob".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "add_post_v2",
            encode_one(bob_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    let bob_post_2 = PostDetailsFromFrontend {
        is_nsfw: true,
        description: "This is a fun video to watch - bob2".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234bob2".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "add_post_v2",
            encode_one(bob_post_2).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // Call post cache canister to get the home feed posts
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 4);
    assert_eq!(posts[0].post_id, 0);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 0);
    assert_eq!(posts[3].post_id, 1);

    // Upgrade individual canister

    let individual_template_wasm_bytes = individual_template_canister_wasm();

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(alice_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    let res = pic.upgrade_canister(
        alice_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );
    if let Err(e) = res {
        panic!("Error: {:?}", e);
    }

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(bob_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    let res = pic.upgrade_canister(
        bob_individual_template_canister_id,
        individual_template_wasm_bytes,
        individual_template_args_bytes,
        None,
    );
    if let Err(e) = res {
        panic!("Error: {:?}", e);
    }

    // Delete the post
    let res = pic.update_call(
        bob_individual_template_canister_id,
        admin_principal_id,
        "delete_post_temp",
        encode_one(1 as u64).unwrap(),
    );

    // Check if post is deleted

    let res = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_individual_post_details_by_id",
            encode_one(1 as u64).unwrap(),
        )
        .map(|reply_payload| {
            let post: Result<_, String> = match reply_payload {
                WasmResult::Reply(payload) => panic!("\n🛑 Expected get_post to fail\n"),
                _ => Ok(()),
            };
        });

    // Call post cache canister to get the home feed posts - old
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 4);
    assert_eq!(posts[0].post_id, 0);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 0);
    assert_eq!(posts[3].post_id, 1);

    // Upgrade post cache canister

    let post_cache_wasm_bytes = post_cache_canister_wasm();

    let post_cache_args = PostCacheInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        upgrade_version_number: Some(1),
        version: "1".to_string(),
    };
    let post_cache_args_bytes = encode_one(post_cache_args).unwrap();

    let res = pic.upgrade_canister(
        post_cache_canister_id,
        post_cache_wasm_bytes.clone(),
        post_cache_args_bytes.clone(),
        None,
    );
    if let Err(e) = res {
        panic!("Error: {:?}", e);
    }

    pic.advance_time(Duration::from_secs(5));
    pic.tick();

    // Call post cache canister to get the home feed posts - old
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 4);
    assert_eq!(posts[0].post_id, 0);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 0);
    assert_eq!(posts[3].post_id, 1);

    // Call post cache canister to get the home feed posts
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed_cursor",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].post_id, 0);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 0);

    // Call post cache canister to get the hot or not feed posts
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItemV1>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].post_id, 0);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 0);
    assert_eq!(posts[2].is_nsfw, true);

    // Bob creates a post

    let bob_post_2 = PostDetailsFromFrontend {
        is_nsfw: true,
        description: "This is a fun video to watch - bob2".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234bob2".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "add_post_v2",
            encode_one(bob_post_2).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    // Call post cache canister to get the hot or not feed posts
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItemV1>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 4);
    assert_eq!(posts[0].post_id, 0);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 0);
    assert_eq!(posts[3].post_id, 1);

    // Call post cache canister to get the home feed posts - old function to verify backward compatibitlity
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 4);
    assert_eq!(posts[0].post_id, 0);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 0);
    assert_eq!(posts[3].post_id, 1);

    // Call post cache canister to get the hot or not feed posts - old function to verify backward compatibitlity
    let res = pic
        .query_call(
            post_cache_canister_id,
            bob_principal_id,
            "get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_posts failed\n"),
            };
            posts
        })
        .unwrap();

    let posts = res.unwrap();
    assert_eq!(posts.len(), 4);
    assert_eq!(posts[0].post_id, 0);
    assert_eq!(posts[1].post_id, 1);
    assert_eq!(posts[2].post_id, 0);
    assert_eq!(posts[3].post_id, 1);
}

fn old_individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(OLD_INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn old_post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(OLD_POST_CACHE_WASM_PATH).unwrap()
}

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}
