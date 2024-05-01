use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime},
};

use candid::{encode_args, encode_one, CandidType, Principal};
use ic_cdk::api::management_canister::main::{CanisterId, CanisterSettings};
use ic_ledger_types::{AccountIdentifier, BlockIndex, Tokens, DEFAULT_SUBACCOUNT};
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::PlaceBetArg,
            error::BetOnCurrentlyViewingPostError,
            hot_or_not::{BetDirection, BettingStatus},
            post::PostDetailsFromFrontend,
            profile::UserProfileDetailsForFrontend,
        },
        platform_orchestrator::types::args::PlatformOrchestratorInitArgs,
        post_cache::types::arg::PostCacheInitArgs,
        user_index::types::{args::UserIndexInitArgs, RecycleStatus},
    },
    common::types::{known_principal::KnownPrincipalType, wasm::WasmType},
    constant::{NNS_CYCLE_MINTING_CANISTER, NNS_LEDGER_CANISTER_ID},
};
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
    v1::CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
};

const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";
const POST_CACHE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/post_cache.wasm.gz";

const USER_INDEX_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/user_index.wasm.gz";
const PF_ORCH_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/platform_orchestrator.wasm.gz";

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

fn user_index_canister_wasm() -> Vec<u8> {
    std::fs::read(USER_INDEX_WASM_PATH).unwrap()
}

fn post_cache_canister_wasm() -> Vec<u8> {
    std::fs::read(POST_CACHE_WASM_PATH).unwrap()
}
fn pf_orch_canister_wasm() -> Vec<u8> {
    std::fs::read(PF_ORCH_WASM_PATH).unwrap()
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

#[derive(CandidType)]
struct NnsLedgerCanisterInitPayload {
    minting_account: String,
    initial_values: HashMap<String, Tokens>,
    send_whitelist: HashSet<CanisterId>,
    transfer_fee: Option<Tokens>,
}

// TODO: remove this when removing the update_last_access_time API from PF Orch

#[test]
fn update_canisters_last_access_time_test() {
    let pic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .with_application_subnet()
        .with_system_subnet()
        .build();
    let admin_principal_id = get_mock_user_charlie_principal_id();
    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();

    let application_subnets = pic.topology().get_app_subnets();

    let platform_canister_id = pic.create_canister_with_settings(
        Some(admin_principal_id),
        Some(CanisterSettings {
            controllers: Some(vec![admin_principal_id]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
    );
    pic.add_cycles(
        platform_canister_id,
        CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    );
    let platform_orchestrator_wasm = pf_orch_canister_wasm();
    let subnet_orchestrator_canister_wasm = user_index_canister_wasm();
    let individual_user_template = individual_template_canister_wasm();
    let platform_orchestrator_init_args = PlatformOrchestratorInitArgs {
        version: "v1.0.0".into(),
    };
    pic.install_canister(
        platform_canister_id,
        platform_orchestrator_wasm.clone(),
        candid::encode_one(platform_orchestrator_init_args).unwrap(),
        Some(admin_principal_id),
    );
    for _ in 0..30 {
        pic.tick()
    }

    pic.update_call(
        platform_canister_id,
        admin_principal_id,
        "upload_wasms",
        candid::encode_args((
            WasmType::SubnetOrchestratorWasm,
            subnet_orchestrator_canister_wasm.to_vec(),
        ))
        .unwrap(),
    )
    .unwrap();
    pic.update_call(
        platform_canister_id,
        admin_principal_id,
        "upload_wasms",
        candid::encode_args((
            WasmType::IndividualUserWasm,
            individual_user_template.to_vec(),
        ))
        .unwrap(),
    )
    .unwrap();
    pic.add_cycles(platform_canister_id, 10_000_000_000_000_000);

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

    //Ledger Canister
    let minting_account = AccountIdentifier::new(&admin_principal_id, &DEFAULT_SUBACCOUNT);
    let ledger_canister_wasm = include_bytes!("../../ledger-canister.wasm");
    let ledger_canister_id = pic
        .create_canister_with_id(
            Some(admin_principal_id),
            None,
            Principal::from_text(NNS_LEDGER_CANISTER_ID).unwrap(),
        )
        .unwrap();
    let icp_ledger_init_args = NnsLedgerCanisterInitPayload {
        minting_account: minting_account.to_string(),
        initial_values: HashMap::new(),
        send_whitelist: HashSet::new(),
        transfer_fee: Some(Tokens::from_e8s(10_000)),
    };
    pic.install_canister(
        ledger_canister_id,
        ledger_canister_wasm.into(),
        candid::encode_one(icp_ledger_init_args).unwrap(),
        Some(admin_principal_id),
    );

    //Cycle Minting Canister
    let cycle_minting_canister_wasm = include_bytes!("../../cycles-minting-canister.wasm");
    let cycle_minting_canister_id = pic
        .create_canister_with_id(
            Some(admin_principal_id),
            None,
            Principal::from_text(NNS_CYCLE_MINTING_CANISTER).unwrap(),
        )
        .unwrap();
    pic.add_cycles(
        cycle_minting_canister_id,
        CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    );
    let cycles_minting_canister_init_args = CyclesMintingCanisterInitPayload {
        ledger_canister_id: ledger_canister_id,
        governance_canister_id: CanisterId::anonymous(),
        minting_account_id: Some(minting_account.to_string()),
        last_purged_notification: Some(0),
    };

    pic.install_canister(
        cycle_minting_canister_id,
        cycle_minting_canister_wasm.into(),
        candid::encode_one(cycles_minting_canister_init_args).unwrap(),
        Some(admin_principal_id),
    );

    let authorized_subnetwork_list_args = AuthorizedSubnetWorks {
        who: Some(platform_canister_id),
        subnets: application_subnets.clone(),
    };
    pic.update_call(
        cycle_minting_canister_id,
        CanisterId::anonymous(),
        "set_authorized_subnetwork_list",
        candid::encode_one(authorized_subnetwork_list_args).unwrap(),
    )
    .unwrap();

    for i in 0..50 {
        pic.tick();
    }

    let charlie_global_admin = get_mock_user_charlie_principal_id();

    pic.update_call(
        platform_canister_id,
        admin_principal_id,
        "add_principal_as_global_admin",
        candid::encode_one(charlie_global_admin).unwrap(),
    )
    .unwrap();

    let user_index_canister_id: Principal = pic
        .update_call(
            platform_canister_id,
            admin_principal_id,
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
        pic.tick();
    }

    // upgrade pf_orch

    let platform_orchestrator_init_args = PlatformOrchestratorInitArgs {
        version: "v1.0.0".into(),
    };
    pic.upgrade_canister(
        platform_canister_id,
        platform_orchestrator_wasm,
        candid::encode_one(platform_orchestrator_init_args).unwrap(),
        Some(admin_principal_id),
    )
    .unwrap();
    for i in 0..20 {
        pic.tick();
    }

    // User Index available details - call get_subnet_available_capacity

    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
            "get_subnet_available_capacity",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let subnet_capacity: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_subnet_available_capacity failed\n"),
            };
            subnet_capacity
        })
        .unwrap();
    println!("Avail capacity: {:?}", res);

    // call get_subnet_backup_capacity

    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
            "get_subnet_backup_capacity",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let subnet_capacity: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_subnet_backup_capacity failed\n"),
            };
            subnet_capacity
        })
        .unwrap();
    println!("Backup capacity: {:?}", res);

    // create user canisters

    let alice_individual_template_canister_id = pic
        .update_call(
            user_index_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Principal = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
            };
            result
        })
        .unwrap();
    println!("res1: {:?}", alice_individual_template_canister_id);

    let bob_individual_template_canister_id = pic
        .update_call(
            user_index_canister_id,
            bob_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Principal = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
            };
            result
        })
        .unwrap();
    println!("res2: {:?}", bob_individual_template_canister_id);

    let dan_individual_template_canister_id = pic
        .update_call(
            user_index_canister_id,
            dan_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Principal = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer failed\n"),
            };
            result
        })
        .unwrap();
    println!("res3: {:?}", dan_individual_template_canister_id);

    pic.add_cycles(alice_individual_template_canister_id, 2_000_000_000_000);
    pic.add_cycles(bob_individual_template_canister_id, 2_000_000_000_000);
    pic.add_cycles(dan_individual_template_canister_id, 2_000_000_000_000);

    // Call get_last_canister_functionality_access_time for all users

    let res1 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();
    println!("res1: {:?}", res1);

    let res2 = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();
    println!("res2: {:?}", res2);

    let res3 = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();
    println!("res3: {:?}", res3);

    // Forward timer
    pic.advance_time(Duration::from_secs(60 * 60));
    for _ in 0..5 {
        pic.tick();
    }

    // Call update_canisters_last_functionality_access_time on pf orch

    let res = pic
        .update_call(
            platform_canister_id,
            admin_principal_id,
            "update_canisters_last_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_canisters_last_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();
    println!("res: {:?}", res);

    for _ in 0..5 {
        pic.tick();
    }

    // Call get_last_canister_functionality_access_time for all users

    let res11 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();
    println!("res11: {:?}", res11);
    assert!(res11 > res1);

    let res22 = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();
    println!("res22: {:?}", res22);
    assert!(res22 > res2);

    let res33 = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let result: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            result
        })
        .unwrap();
    println!("res33: {:?}", res33);
    assert!(res33 > res3);
}
