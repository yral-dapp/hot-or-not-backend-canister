use std::collections::HashMap;

use candid::{encode_args, encode_one, Principal};
use ic_cdk::println;
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::{IndividualUserTemplateInitArgs, PlaceBetArg},
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{BetDirection, BettingStatus},
            post::PostDetailsFromFrontend,
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id,
    get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id,
    get_mock_user_dan_principal_id,
};

use shared_utils::common::types::utility_token::token_event::NewSlotType;

const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}

#[test]
fn test_receive_call_from_another_canister() {
    let pic = PocketIc::new();

    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();
    let admin_principal_id = get_mock_user_charlie_principal_id();

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );
    known_prinicipal_values.insert(KnownPrincipalType::CanisterIdUserIndex, admin_principal_id);

    let post_cache_wasm_bytes = post_cache_canister_wasm();
    let post_cache_args = PostCacheInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        upgrade_version_number: Some(1),
        version: "1".to_string(),
    };
    let post_cache_args_bytes = encode_one(post_cache_args).unwrap();
    pic.install_canister(
        post_cache_canister_id,
        post_cache_wasm_bytes,
        post_cache_args_bytes,
        None,
    );

    // Individual template canisters
    let individual_template_wasm_bytes = individual_template_canister_wasm();

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

    // Init individual template canister - dan

    let dan_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(dan_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(dan_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        dan_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    // Init charlie individual template canister

    let charlie_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(charlie_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(admin_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        charlie_individual_template_canister_id,
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
    let res1 = pic
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
    let res2 = pic
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

    // Top up Bob's account
    let reward = pic.update_call(
        bob_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // Top up Dan's account
    let reward = pic.update_call(
        dan_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // Top up Charlie's account
    let reward = pic.update_call(
        charlie_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // Bob places bet on Alice post 1
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };

    let payload: (PlaceBetArg, Principal) = (bob_place_bet_arg.clone(), bob_principal_id);

    let bet_status = pic
        .update_call(
            alice_individual_template_canister_id,
            bob_place_bet_arg.post_canister_id,
            "receive_bet_from_bet_makers_canister",
            encode_args(payload).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => {
                        let payload_result = candid::decode_one(&payload);
                        dbg!(&payload_result);
                        payload_result.unwrap()
                    }
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();

    println!("{:?}", &bet_status);

    assert!(matches!(
        bet_status,
        BettingStatus::BettingOpen {
            number_of_participants: 1,
            ongoing_slot: NewSlotType(1),
            ongoing_room: 1,
            has_this_user_participated_in_this_post: Some(true),
            ..
        }
    ));

    // todo assert that timer is set here.
}
