use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
    vec,
};

use candid::{CandidType, Encode, Principal};
use ic_cdk::api::{management_canister::provisional::CanisterSettings, time};
use ic_ledger_types::{AccountIdentifier, BlockIndex, Tokens, DEFAULT_SUBACCOUNT};
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::{
        individual_user_template,
        platform_orchestrator::{
            self,
            types::args::{PlatformOrchestratorInitArgs, UpgradeCanisterArg},
        },
        post_cache::types::arg::PostCacheInitArgs,
    },
    common::{
        types::{
            known_principal::{self, KnownPrincipalMap, KnownPrincipalType},
            wasm::WasmType,
        },
        utils::system_time,
    },
    constant::{NNS_CYCLE_MINTING_CANISTER, NNS_LEDGER_CANISTER_ID, YRAL_POST_CACHE_CANISTER_ID},
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
        get_mock_user_charlie_principal_id, v1::CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    },
};

use ic_sns_wasm::init::SnsWasmCanisterInitPayload;
pub type CanisterId = Principal;

pub const SNS_WASM_W_PRINCIPAL_ID: &'static str = "qaa6y-5yaaa-aaaaa-aaafa-cai";

#[test]
fn creator_dao_tests() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let charlie_global_admin = get_mock_user_charlie_principal_id();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "add_principal_as_global_admin",
            candid::encode_one(charlie_global_admin).unwrap(),
        )
        .unwrap();

    let subnet_orchestrator_canister_id: Principal = pocket_ic
        .update_call(
            platform_canister_id,
            charlie_global_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[1]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for i in 0..50 {
        pocket_ic.tick();
    }

    let alice_principal = get_mock_user_alice_principal_id();
    let alice_cannister_id: Principal = pocket_ic.update_call(
        subnet_orchestrator_canister_id,
        alice_principal,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let response: Principal = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    })
    .unwrap();

    let sns_wasm_w_canister_wasm = include_bytes!("../../../../../sns-wasm-canister.wasm");

    let sns_wasm_w_canister_id = Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap();

    pocket_ic.create_canister_with_id(
        Some(super_admin),
        None,
        Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap(),
    );

    let sns_wasm_canister_init_payload = SnsWasmCanisterInitPayload {
        sns_subnet_ids: vec![],
        access_controls_enabled: false,
        allowed_principals: vec![],
    };

    pocket_ic.install_canister(
        sns_wasm_w_canister_id,
        sns_wasm_w_canister_wasm.to_vec(),
        Encode!(&sns_wasm_canister_init_payload).unwrap(),
        Some(super_admin),
    );

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            Principal::anonymous(),
            "get_latest_sns_version_pretty".into(),
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let response: HashMap<String, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();

    ic_cdk::println!("🧪 HASHMAP {:?}", res);
    assert_eq!(res.len(), 0);
    // let sns_init = SnsInitPayload {

    // }

    // pocket_ic.update_call(
    //     alice_cannister_id,
    //     alice_principal_id,
    //     "deploy_cdao_sns",

    // )
}
