use candid::{CandidType, Principal};
use ic_cdk::api::management_canister::provisional::CanisterSettings;
use ic_ledger_types::{AccountIdentifier, BlockIndex, Tokens, DEFAULT_SUBACCOUNT};
use ic_test_state_machine_client::WasmResult as StateMachineWasmResult;
use pocket_ic::{PocketIcBuilder, WasmResult as PocketICWasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{migration::MigrationErrors, post::Post},
        platform_orchestrator::types::args::PlatformOrchestratorInitArgs,
    },
    common::types::{known_principal::KnownPrincipalType, wasm::WasmType},
    constant::{NNS_CYCLE_MINTING_CANISTER, NNS_LEDGER_CANISTER_ID},
};
use std::collections::{BTreeMap, HashMap, HashSet};
use test_utils::setup::{
    env::{
        pocket_ic_env::get_new_pocket_ic_env,
        v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
    },
    test_constants::{
        get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id, v1::CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    },
};

#[test]
fn error_when_owner_is_not_caller() {
    let state_machine = get_new_state_machine();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();
    let anonymous_principal_id = Principal::anonymous();
    let alice_principal_id = get_mock_user_alice_principal_id();

    let alice_canister_id = state_machine.update_call(
        *user_index_canister_id,
        alice_principal_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let alice_canister_id: Principal = match reply_payload {
            StateMachineWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        alice_canister_id
    }).unwrap();

    let error_owner_is_not_caller = state_machine
        .update_call(
            alice_canister_id,
            anonymous_principal_id,
            "transfer_tokens_and_posts",
            candid::encode_args((alice_principal_id, alice_canister_id)).unwrap(),
        )
        .map(|reply_payload| {
            let error_owner_is_not_caller: Result<(), MigrationErrors> = match reply_payload {
                StateMachineWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 transfer_tokens_and_posts failed\n"),
            };
            error_owner_is_not_caller
        })
        .unwrap();

    assert_eq!(
        error_owner_is_not_caller,
        Err(MigrationErrors::Unauthorized)
    );
}

#[test]
fn error_when_receiver_profiler_owner_is_not_receiver_caller() {
    let state_machine = get_new_state_machine();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&state_machine);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();
    let anonymous_principal_id = Principal::anonymous();
    let alice_principal_id = get_mock_user_alice_principal_id();

    let alice_canister_id = state_machine.update_call(
        *user_index_canister_id,
        alice_principal_id,
        "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
        candid::encode_one(()).unwrap(),
    ).map(|reply_payload| {
        let alice_canister_id: Principal = match reply_payload {
            StateMachineWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
        };
        alice_canister_id
    }).unwrap();

    let posts: BTreeMap<u64, Post> = BTreeMap::new();

    let error_owner_is_not_caller = state_machine
        .update_call(
            alice_canister_id,
            anonymous_principal_id,
            "receive_data_from_hotornot",
            candid::encode_args((1000u64, alice_principal_id, posts)).unwrap(),
        )
        .map(|reply_payload| {
            let error_owner_is_not_caller: Result<String, String> = match reply_payload {
                StateMachineWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 receive_data_from_hotornot failed\n"),
            };
            error_owner_is_not_caller
        })
        .unwrap();

    assert_eq!(
        error_owner_is_not_caller,
        Err("Unauthorized caller".to_owned())
    );
}

#[test]
fn migrate_posts_and_tokens_from_hotornot_to_yral_account_successfully() {
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

    let hot_or_not_subnet_orchestrator_canister_id: Principal = pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[0]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    let yral_subnet_orchestrator_canister_id: Principal = pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[1]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for _ in 0..30 {
        pocket_ic.tick();
    }

    let post_cache_canister_id = Principal::anonymous();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "update_global_known_principal",
            candid::encode_args((
                KnownPrincipalType::CanisterIdHotOrNotSubnetOrchestrator,
                hot_or_not_subnet_orchestrator_canister_id,
            ))
            .unwrap(),
        )
        .unwrap();

    for _ in 0..30 {
        pocket_ic.tick();
    }

    //get alice canister-id
    let alice_principal_id = get_mock_user_alice_principal_id();
    let alice_canister_id: Principal = pocket_ic.update_call(hot_or_not_subnet_orchestrator_canister_id, alice_principal_id, "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer", candid::encode_one(()).unwrap())
    .map(|res| {
        let canister_id: Principal = match res {
            PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("Canister call failed")
        };
        canister_id
    })
    .unwrap();

    let bob_principal_id = get_mock_user_bob_principal_id();
    let bob_canister_id: Principal = pocket_ic.update_call(yral_subnet_orchestrator_canister_id, bob_principal_id, "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer", candid::encode_one(()).unwrap())
    .map(|res| {
        let canister_id: Principal = match res {
            PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("Canister call failed")
        };
        canister_id
    })
    .unwrap();

    //update subnet known principal
    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "update_subnet_known_principal",
            candid::encode_args((
                hot_or_not_subnet_orchestrator_canister_id,
                KnownPrincipalType::CanisterIdPostCache,
                post_cache_canister_id,
            ))
            .unwrap(),
        )
        .map(|res| {
            let update_res: Result<String, String> = match res {
                PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("update subnet_known_principal"),
            };
            update_res
        })
        .unwrap()
        .unwrap();

    for _ in 0..30 {
        pocket_ic.tick()
    }

    // transfer token
    let success = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "transfer_tokens_and_posts",
            candid::encode_args((bob_principal_id, bob_canister_id)).unwrap(),
        )
        .map(|reply_payload| {
            let success: Result<(), MigrationErrors> = match reply_payload {
                PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 transfer_tokens_and_posts failed\n"),
            };
            success
        })
        .unwrap();

    assert_eq!(success, Ok(()));

    for _ in 0..10 {
        pocket_ic.tick();
    }

    let bob_utility_balance = pocket_ic
        .query_call(
            bob_canister_id,
            Principal::anonymous(),
            "get_utility_token_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let balance: u64 = match reply_payload {
                PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 transfer_tokens_and_posts failed\n"),
            };
            balance
        })
        .unwrap();

    assert_eq!(bob_utility_balance, 2000);
}

// #[test]
// fn error_when_receiver_is_already_migrated() {}

pub type CanisterId = Principal;

#[derive(CandidType)]
struct NnsLedgerCanisterInitPayload {
    minting_account: String,
    initial_values: HashMap<String, Tokens>,
    send_whitelist: HashSet<CanisterId>,
    transfer_fee: Option<Tokens>,
}

#[derive(CandidType)]
struct CyclesMintingCanisterInitPayload {
    ledger_canister_id: CanisterId,
    governance_canister_id: CanisterId,
    minting_account_id: Option<String>,
    last_purged_notification: Option<BlockIndex>,
}

#[derive(CandidType)]
struct AuthorizedSubnetWorks {
    who: Option<Principal>,
    subnets: Vec<Principal>,
}
