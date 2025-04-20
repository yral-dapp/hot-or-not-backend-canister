use std::time::Duration;

use candid::{encode_one, Principal};
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::{IndividualUserTemplateInitArgs, PlaceBetArg},
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BetDirection, BettingStatus},
        post::PostDetailsFromFrontend,
        pump_n_dump::BalanceInfo,
    },
    common::types::known_principal::KnownPrincipalType,
    constant::GDOLLR_TO_E8S,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_bob_principal_id},
};

#[test]
pub fn test_hot_or_not_bets_placed_using_cents_are_computed_even_after_individual_canister_upgrade()
{
    let (pic, known_principals) = get_new_pocket_ic_env();

    let platform_canister_id = known_principals
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let global_admin = known_principals
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pic.topology().get_app_subnets();

    let subnet_orchestrator_canister_id = pic
        .update_call(
            platform_canister_id,
            global_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[0]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for _ in 0..50 {
        pic.tick()
    }
    let alice_principal_id = get_mock_user_alice_principal_id();

    let alice_canister_id = pic
        .update_call(
            subnet_orchestrator_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();

    let bob_principal_id = get_mock_user_bob_principal_id();

    let bob_canister_id = pic
        .update_call(
            subnet_orchestrator_canister_id,
            bob_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();

    let last_post_1 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let post_id = pic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(last_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    let place_bet_arg = PlaceBetArg {
        post_canister_id: alice_canister_id,
        post_id: post_id,
        bet_amount: 100 * GDOLLR_TO_E8S,
        bet_direction: BetDirection::Hot,
    };

    pic.update_call(
        bob_canister_id,
        global_admin,
        "bet_on_currently_viewing_post_v1",
        encode_one(place_bet_arg).unwrap(),
    )
    .map(|reply_payload| {
        let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> = match reply_payload
        {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 place_bet failed\n"),
        };
        bet_status.unwrap()
    })
    .unwrap();

    let bob_token_balance = pic
        .query_call(
            bob_canister_id,
            bob_principal_id,
            "pd_balance_info",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: BalanceInfo = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap()
        .balance;
    assert_eq!(bob_token_balance, 900 * GDOLLR_TO_E8S as u128);

    pic.advance_time(Duration::from_secs(30 * 60));
    pic.tick();

    let individual_user_template_wasm_module = include_bytes!(
        "../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
    )
    .to_vec();

    let individual_user_template_init_args = IndividualUserTemplateInitArgs {
        known_principal_ids: None,
        profile_owner: None,
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "v2.0.0".to_string(),
        pump_dump_onboarding_reward: None,
    };

    pic.upgrade_canister(
        alice_canister_id,
        individual_user_template_wasm_module,
        encode_one(individual_user_template_init_args).unwrap(),
        Some(subnet_orchestrator_canister_id),
    )
    .unwrap();

    pic.advance_time(Duration::from_secs(65 * 60));
    for _ in 0..20 {
        pic.tick();
    }

    let token_info = pic
        .query_call(
            bob_canister_id,
            bob_principal_id,
            "pd_balance_info",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: BalanceInfo = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();

    assert_eq!(token_info.balance, 1080 * GDOLLR_TO_E8S as u128);
}
