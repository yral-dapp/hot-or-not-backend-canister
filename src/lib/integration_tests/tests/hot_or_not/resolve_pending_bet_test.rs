use std::{collections::HashMap, time::Duration};

use candid::{encode_args, encode_one, Principal};
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::{IndividualUserTemplateInitArgs, PlaceBetArg},
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{BetDirection, BettingStatus, GlobalBetId, GlobalRoomId, PlacedBetDetail},
            post::PostDetailsFromFrontend,
            profile::UserProfileDetailsForFrontend,
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
    get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id,
    get_mock_user_dan_principal_id,
};

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
// ONLY local testing because simulating a hung timer requires to expose additional functions
// insert_in_all_hot_or_not_bets_placed , insert_in_bet_details_map  -> src/canister/individual_user_template/src/api/hot_or_not_bet/resolve_pending_bets.rs
// #[cfg(feature = "integration_tests")]
fn resolve_pending_bet_test() {
    let pic = PocketIc::new();

    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();
    let admin_principal_id = get_mock_user_charlie_principal_id();

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let mut known_principal_values = HashMap::new();
    known_principal_values.insert(
        KnownPrincipalType::CanisterIdPostCache,
        post_cache_canister_id,
    );
    known_principal_values.insert(
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

        // Top up Bob's account
    let reward = pic.update_call(
        bob_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // simulate a hung timer 

    // insert in bob_canister.all_hot_or_not_bet_placed.insert(alice_canister_id, post_id)
    let arguments = (alice_individual_template_canister_id, res1, PlacedBetDetail{
        canister_id: alice_individual_template_canister_id,
        post_id: res1,
        slot_id: 1,
        room_id: 1,
        amount_bet: 100,
        bet_direction: BetDirection::Hot,
        outcome_received: BetOutcomeForBetMaker::default(),
        bet_placed_at: pic.get_time()
    });


    pic.update_call(bob_individual_template_canister_id, bob_principal_id, "insert_in_all_hot_or_not_bets_placed", encode_args(arguments).unwrap());
    
    // insert in alice_canister.bet_details_map.insert(global_bet_id, bob_principal_id)

    let bdm_value = BetDetails {
        amount: 100,
        bet_direction: BetDirection::Hot,
        payout: BetPayout::Calculated(180),
        bet_maker_canister_id: bob_individual_template_canister_id,
    };
        
    let bdm_args = (GlobalBetId(GlobalRoomId(res1, 1_u8, 1_u64), alice_principal_id), bdm_value);
    
    pic.update_call(alice_individual_template_canister_id, bob_principal_id, "insert_in_bet_details_map", encode_one(bdm_args).unwrap());

    // check if bob.all_hot_or_not_bets_placed.get(alice_canister_id, post_id)
    //  use get_hot_or_not_bets_placed_by_this_profile_with_pagination
    let bob_all_hot_or_not_bets_placed = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_hot_or_not_bets_placed_by_this_profile_with_pagination",
            encode_one(0).unwrap(),
        )
        .map(|reply_payload| {
            let bets: Vec<PlacedBetDetail> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_hot_or_not_bets_placed_by_this_profile_with_pagination failed\n"),
            };
            bets
        })
        .unwrap();

    ic_cdk::println!("Bob all hot or not bets placed: {:?}", bob_all_hot_or_not_bets_placed);
    
    // check if alice.bet_details_map.get(global_bet_id)
    
}