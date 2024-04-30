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

#[test]
fn recycle_canisters_test() {
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

    // User 1 creates posts
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

    // Bob places bet on Alice post 1
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(bob_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Bob places bet on Alice post 2
    let bob_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pic
        .update_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(bob_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 1
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res1,
        bet_amount: 100,
        bet_direction: BetDirection::Hot,
    };
    let bet_status = pic
        .update_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(dan_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Dan places bet on Alice post 2
    let dan_place_bet_arg = PlaceBetArg {
        post_canister_id: alice_individual_template_canister_id,
        post_id: res2,
        bet_amount: 100,
        bet_direction: BetDirection::Not,
    };
    let bet_status = pic
        .update_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "bet_on_currently_viewing_post",
            encode_one(dan_place_bet_arg).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: Result<BettingStatus, BetOnCurrentlyViewingPostError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 place_bet failed\n"),
                };
            bet_status.unwrap()
        })
        .unwrap();
    ic_cdk::println!("Bet status: {:?}", bet_status);

    // Show alice rewards

    let alice_rewards = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice rewards: {:?}", alice_rewards);

    let alice_token_balance = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Alice token balance: {:?}", alice_token_balance);

    // Show bob rewards

    let bob_rewards = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Bob rewards: {:?}", bob_rewards);

    let bob_token_balance = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Bob token balance: {:?}", bob_token_balance);

    // Show dan rewards

    let dan_rewards = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_rewards failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Dan rewards: {:?}", dan_rewards);

    let dan_token_balance = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_utility_token_balance",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let token_balance: u64 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_token_balance failed\n"),
            };
            token_balance
        })
        .unwrap();
    println!("Dan token balance: {:?}", dan_token_balance);

    // upgrade user index canister

    for _ in 0..5 {
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

    // call alice canister - get_last_canister_functionality_access_time
    let res1 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap()
        .unwrap();
    println!("Alice last access time: {:?}", res1);

    // call bob canister - get_last_canister_functionality_access_time
    let res2 = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap()
        .unwrap();
    println!("Bob last access time: {:?}", res2);

    // call dan canister - get_last_canister_functionality_access_time
    let res3 = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap()
        .unwrap();
    println!("Dan last access time: {:?}", res3);

    // Forward timer
    pic.advance_time(Duration::from_secs(32 * 24 * 60 * 60));
    for _ in 0..20 {
        pic.tick();
    }

    // call user_index get_recycle_status
    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
            "get_recycle_status",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let recycle_status: RecycleStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_recycle_status failed\n"),
            };
            recycle_status
        })
        .unwrap();
    println!("Recycle status: {:?}", res);

    // print principal strs
    println!("Admin principal: {:?}", admin_principal_id.to_string());
    println!(
        "User Index principal: {:?}",
        user_index_canister_id.to_string()
    );
    println!(
        "Alice principal: {:?}",
        alice_individual_template_canister_id.to_string()
    );
    println!(
        "Bob principal: {:?}",
        bob_individual_template_canister_id.to_string()
    );
    println!(
        "Dan principal: {:?}",
        dan_individual_template_canister_id.to_string()
    );

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

    // call user_index get_user_index_canister_count
    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
            "get_user_index_canister_count",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let canister_count: usize = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_index_canister_count failed\n"),
            };
            canister_count
        })
        .unwrap();
    println!("ind Canister count: {:?}", res);

    // call user_index recycle_canisters
    // let res = pic
    //     .update_call(
    //         user_index_canister_id,
    //         platform_canister_id,
    //         "recycle_canisters",
    //         encode_one(()).unwrap(),
    //     )
    //     .map(|reply_payload| {
    //         let canister_count: () = match reply_payload {
    //             WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
    //             _ => panic!("\n🛑 recycle_canisters failed\n"),
    //         };
    //         canister_count
    //     })
    //     .unwrap();

    pic.advance_time(Duration::from_secs(24 * 60 * 60));
    for _ in 0..25 {
        pic.tick();
    }

    // call user_index get_recycle_status
    let res = pic
        .query_call(
            user_index_canister_id,
            admin_principal_id,
            "get_recycle_status",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let recycle_status: RecycleStatus = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_recycle_status failed\n"),
            };
            recycle_status
        })
        .unwrap();
    println!("Recycle status: {:?}", res);

    // call bob canister - get_last_canister_functionality_access_time
    let res22 = pic
        .query_call(
            bob_individual_template_canister_id,
            bob_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap();
    assert!(res22.is_err());

    // call dan canister - get_last_canister_functionality_access_time
    let res33 = pic
        .query_call(
            dan_individual_template_canister_id,
            dan_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap();
    assert!(res33.is_err());

    // call alice canister - get_last_canister_functionality_access_time
    let res11 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_last_canister_functionality_access_time",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let last_access_time: Result<SystemTime, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_last_canister_functionality_access_time failed\n"),
            };
            last_access_time
        })
        .unwrap();
    assert!(res11.is_err());
}
