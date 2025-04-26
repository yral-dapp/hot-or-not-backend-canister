use std::{collections::HashMap, time::Duration};

use candid::encode_one;
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::
        individual_user_template::types::
            arg::IndividualUserTemplateInitArgs
        
    ,
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id, get_mock_user_charlie_principal_id,
};

// use shared_utils::canister_specific::individual_user_template::types::arg::update_token_balance_before_bet_happens;

const OLD_INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template_main_branch.wasm.gz";
const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";

fn old_individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(OLD_INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

#[cfg(feature = "excessive_tokens")]
#[test]
fn test_migrate_excessive_tokens() {
    let pic = PocketIc::new();

    let alice_principal_id = get_mock_user_alice_principal_id();
    let admin_principal_id = get_mock_user_charlie_principal_id();

    // let post_cache_canister_id = pic.create_canister();
    // pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    let mut known_prinicipal_values = HashMap::new();
    known_prinicipal_values.insert(
        KnownPrincipalType::UserIdGlobalSuperAdmin,
        admin_principal_id,
    );
    known_prinicipal_values.insert(KnownPrincipalType::CanisterIdUserIndex, admin_principal_id);

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

    // Topup Alice's account
    let reward = pic.update_call(
        alice_individual_template_canister_id,
        admin_principal_id,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    // from main branch, deposit many tokens,

    // this a hack to increase the token balance beyond the limit.
    let bet_amount = 18_00_00_00_00_00_00_00_00_00 + 1 as u64;

    let _update = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "update_token_balance_before_bet_happens",
            encode_one(bet_amount).unwrap(),
        )
        .map(|reply_payload| ic_cdk::println!("{reply_payload:?}"));

    // load new wasm
    // Upgrade the individual template canister to the new version

    let individual_template_wasm_bytes = individual_template_canister_wasm();

    // Alice canister

    let individual_template_args = IndividualUserTemplateInitArgs {
        known_principal_ids: Some(known_prinicipal_values.clone()),
        profile_owner: Some(alice_principal_id),
        upgrade_version_number: None,
        url_to_send_canister_metrics_to: None,
        version: "1".to_string(),
    };
    let individual_template_args_bytes = encode_one(individual_template_args).unwrap();

    pic.advance_time(Duration::from_secs(20));
    pic.tick();
    pic.tick();
    pic.tick();

    pic.upgrade_canister(
        alice_individual_template_canister_id,
        individual_template_wasm_bytes.clone(),
        individual_template_args_bytes,
        None,
    )
    .unwrap();

    // assert the result from the post ugprade hook.

    let utility_token = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
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

    ic_cdk::println!("UTILITY TOKENS NOW ARE: {utility_token}");
}

#[cfg(feature = "excessive_tokens")]
#[test]
fn test_migrate_excessive_tokens_no_change() {}
