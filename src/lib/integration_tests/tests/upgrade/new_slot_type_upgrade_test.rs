use std::{collections::HashMap, thread, time::Duration};

use candid::{encode_one, CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::main::CanisterId;
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::{IndividualUserTemplateInitArgs, PlaceBetArg},
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{BetDirection, BettingStatus, PlacedBetDetail},
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
    get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
    get_mock_user_dholak_principal_id,
};

const OLD_POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache_main_branch.wasm.gz";
const OLD_INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template_main_branch.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";
const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";

// #[cfg(feature = "feed_filter_upgrade_test")]
#[test]
// #[ignore = "New Slot Type Upgrade to be tested only locally"]
fn new_slot_type_upgrade_test() {
    let pic = PocketIc::new();

    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let admin_principal_id = get_mock_user_charlie_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();
    let dholak_principal_id = get_mock_user_dholak_principal_id();

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

    let post_cache_wasm_bytes = old_post_cache_canister_wasm();

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

    let individual_template_wasm_bytes = old_individual_template_canister_wasm();

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

    // Init individual template canister - dholak

    let dholak_individual_template_canister_id = pic.create_canister();
    pic.add_cycles(dholak_individual_template_canister_id, 2_000_000_000_000);

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(dholak_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.install_canister(
        dholak_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    );

    //  --------- TOP UP account START ---------
    // Top up Alice's account
    let reward = pic.update_call(
        alice_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

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

    // Top up Dholak's account
    let reward = pic.update_call(
        dholak_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );
    //  --------------- TOP UP account DONE ----------------

    //  --------- Create posts START ---------

    // Alice creates a post
    let alice_posts = create_posts_for_user(
        &pic,
        5,
        alice_individual_template_canister_id,
        alice_principal_id,
    );

    // Bob creates a post
    let bob_posts = create_posts_for_user(
        &pic,
        5,
        bob_individual_template_canister_id,
        bob_principal_id,
    );

    // Dan creates a post
    let dan_posts = create_posts_for_user(
        &pic,
        5,
        dan_individual_template_canister_id,
        dan_principal_id,
    );

    //  --------- Create posts DONE ---------

    //  --------------- BETTING START ----------------

    // Bob places bet on Alice post 1
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: alice_posts[0],
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = user_bets_on_post(
        &pic,
        bob_principal_id,
        bob_individual_template_canister_id,
        bob_place_bet_arg,
    );
    println!("Bet status: {:?}", bet_status);

    // Bob places bet on Alice post 2
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: alice_posts[1],
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = user_bets_on_post(
        &pic,
        bob_principal_id,
        bob_individual_template_canister_id,
        bob_place_bet_arg,
    );
    println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 1
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: alice_posts[0],
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = user_bets_on_post(
        &pic,
        dan_principal_id,
        dan_individual_template_canister_id,
        dan_place_bet_arg,
    );
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 2
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: alice_posts[1],
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = user_bets_on_post(
        &pic,
        dan_principal_id,
        dan_individual_template_canister_id,
        dan_place_bet_arg,
    );
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dholak places bet on Alice post 1
    let dholak_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: alice_posts[0],
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = user_bets_on_post(
        &pic,
        dholak_principal_id,
        dholak_individual_template_canister_id,
        dholak_place_bet_arg,
    );
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dholak places bet on Alice post 2
    let dholak_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: alice_posts[1],
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = user_bets_on_post(
        &pic,
        dholak_principal_id,
        dholak_individual_template_canister_id,
        dholak_place_bet_arg,
    );
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // -------- BETTING END --------

    // // Call post cache canister to get the home feed posts

    // let res = pic
    //     .query_call(
    //         post_cache_canister_id,
    //         bob_principal_id,
    //         "get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed_cursor",
    //         candid::encode_args((0_u64, 10_u64)).unwrap(),
    //     )
    //     .map(|reply_payload| {
    //         let posts: Result<Vec<PostScoreIndexItem>, TopPostsFetchError> = match reply_payload {
    //             WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
    //             _ => panic!("\n🛑 get_posts failed\n"),
    //         };
    //         posts
    //     })
    //     .unwrap();

    // let posts = res.unwrap();
    // assert_eq!(posts.len(), 15);

    let bob_hon_bets = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_hot_or_not_bets_placed_by_this_profile_with_pagination",
            candid::encode_args((0_usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Vec<PlacedBetDetail> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();

    dbg!(&bob_hon_bets);
    // upgrade the canister with new wasm
    let individual_template_wasm_bytes = individual_template_canister_wasm();

    // upgrade canister for users

    upgrade_canister_for_user(
        &pic,
        alice_principal_id,
        alice_individual_template_canister_id,
        known_prinicipal_values.clone(),
        individual_template_wasm_bytes.clone(),
    );
    upgrade_canister_for_user(
        &pic,
        bob_principal_id,
        bob_individual_template_canister_id,
        known_prinicipal_values.clone(),
        individual_template_wasm_bytes.clone(),
    );
    upgrade_canister_for_user(
        &pic,
        dan_principal_id,
        dan_individual_template_canister_id,
        known_prinicipal_values.clone(),
        individual_template_wasm_bytes.clone(),
    );
    upgrade_canister_for_user(
        &pic,
        dholak_principal_id,
        dholak_individual_template_canister_id,
        known_prinicipal_values.clone(),
        individual_template_wasm_bytes.clone(),
    );

    let bob_hon_bets_after_upgrade = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_hot_or_not_bets_placed_by_this_profile_with_pagination",
            candid::encode_args((0_usize,)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Vec<PlacedBetDetail> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();

    dbg!(&bob_hon_bets_after_upgrade);
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

fn upgrade_canister_for_user(
    pic: &PocketIc,
    user_principal_id: Principal,
    user_canister_id: CanisterId,
    known_prinicipal_values: HashMap<KnownPrincipalType, CanisterId>,
    individual_template_wasm_bytes: Vec<u8>,
) {
    // let individual_template_wasm_bytes = individual_template_canister_wasm();

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values),
        profile_owner: Some(user_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    let res = pic.upgrade_canister(
        user_canister_id,
        individual_template_wasm_bytes,
        individual_template_args_bytes,
        None,
    );
    if let Err(e) = res {
        panic!("Error: {:?}", e);
    }
}

fn user_bets_on_post(
    pic: &PocketIc,
    bet_maker_principal_id: Principal,
    bet_maker_individual_template_canister_id: CanisterId,
    bet_maker_place_bet_arg: PlaceBetArg,
) -> BettingStatus {
    pic.update_call(
        bet_maker_individual_template_canister_id,
        bet_maker_principal_id,
        "bet_on_currently_viewing_post",
        encode_one(bet_maker_place_bet_arg).unwrap(),
    )
    .map(|reply_payload| {
        let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> = match reply_payload
        {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 place_bet failed\n"),
        };
        bet_status.unwrap()
    })
    .unwrap()
}
fn create_posts_for_user(
    pic: &PocketIc,
    num_posts: u32,
    alice_individual_template_canister_id: CanisterId,
    alice_principal_id: Principal,
) -> Vec<u64> {
    let mut created_post_ids = Vec::new();
    for i in 0..num_posts {
        {
            let alice_post_1 = PostDetailsFromFrontend {
                is_nsfw: false,
                description: format!(
                    "This is a fun video to watch - {} - {:?} ",
                    i, alice_principal_id
                ),
                hashtags: vec!["fun".to_string(), "video".to_string()],
                video_uid: format!("abcd#{}_for_{:?}", i, alice_principal_id),
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

            created_post_ids.push(res);
        }
    }
    created_post_ids
}
