use candid::{CandidType, Principal};
use ic_cdk::api::management_canister::provisional::CanisterSettings;
use ic_ledger_types::{AccountIdentifier, BlockIndex, Tokens, DEFAULT_SUBACCOUNT};
use ic_test_state_machine_client::WasmResult as StateMachineWasmResult;
use pocket_ic::{PocketIcBuilder, WasmResult as PocketICWasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::post::Post,
        platform_orchestrator::types::args::PlatformOrchestratorInitArgs,
    },
    common::types::{known_principal::KnownPrincipalType, wasm::WasmType},
    constant::{NNS_CYCLE_MINTING_CANISTER, NNS_LEDGER_CANISTER_ID},
};
use std::collections::{BTreeMap, HashMap, HashSet};
use test_utils::setup::{
    env::v1::{get_initialized_env_with_provisioned_known_canisters, get_new_state_machine},
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
            candid::encode_one(alice_principal_id).unwrap(),
        )
        .map(|reply_payload| {
            let error_owner_is_not_caller: Result<String, String> = match reply_payload {
                StateMachineWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 transfer_tokens_and_posts failed\n"),
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
    use shared_utils::constant::MockConstantsWrapper;

    let pocket_ic = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .with_application_subnet()
        .with_system_subnet()
        .build();

    let super_admin = get_global_super_admin_principal_id();

    let application_subnets = pocket_ic.topology().get_app_subnets();
    for (i, aps) in application_subnets.iter().enumerate() {
        println!("subnet[{}] {}", i, aps.to_text());
    }

    let platform_canister_id = pocket_ic.create_canister_with_settings(
        Some(super_admin),
        Some(CanisterSettings {
            controllers: Some(vec![super_admin]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
    );
    pocket_ic.add_cycles(
        platform_canister_id,
        CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    );
    let platform_orchestrator_wasm = include_bytes!(
        "../../../../../target/wasm32-unknown-unknown/release/platform_orchestrator.wasm.gz"
    );
    let individual_user_template = include_bytes!(
        "../../../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz"
    );
    let subnet_orchestrator_canister_wasm =
        include_bytes!("../../../../../target/wasm32-unknown-unknown/release/user_index.wasm.gz");
    let platform_orchestrator_init_args = PlatformOrchestratorInitArgs {
        version: "v1.0.0".into(),
    };
    pocket_ic.install_canister(
        platform_canister_id,
        platform_orchestrator_wasm.into(),
        candid::encode_one(platform_orchestrator_init_args).unwrap(),
        Some(super_admin),
    );
    for _ in 0..30 {
        pocket_ic.tick()
    }
    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "upload_wasms",
            candid::encode_args((
                WasmType::SubnetOrchestratorWasm,
                subnet_orchestrator_canister_wasm.to_vec(),
            ))
            .unwrap(),
        )
        .unwrap();
    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "upload_wasms",
            candid::encode_args((
                WasmType::IndividualUserWasm,
                individual_user_template.to_vec(),
            ))
            .unwrap(),
        )
        .unwrap();
    pocket_ic.add_cycles(platform_canister_id, 10_000_000_000_000_000);

    //Ledger Canister
    let minting_account = AccountIdentifier::new(&super_admin, &DEFAULT_SUBACCOUNT);
    let ledger_canister_wasm = include_bytes!("../../ledger-canister.wasm");
    let ledger_canister_id = pocket_ic
        .create_canister_with_id(
            Some(super_admin),
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
    pocket_ic.install_canister(
        ledger_canister_id,
        ledger_canister_wasm.into(),
        candid::encode_one(icp_ledger_init_args).unwrap(),
        Some(super_admin),
    );

    //Cycle Minting Canister
    let cycle_minting_canister_wasm = include_bytes!("../../cycles-minting-canister.wasm");
    let cycle_minting_canister_id = pocket_ic
        .create_canister_with_id(
            Some(super_admin),
            None,
            Principal::from_text(NNS_CYCLE_MINTING_CANISTER).unwrap(),
        )
        .unwrap();
    pocket_ic.add_cycles(
        cycle_minting_canister_id,
        CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS,
    );
    let cycles_minting_canister_init_args = CyclesMintingCanisterInitPayload {
        ledger_canister_id,
        governance_canister_id: CanisterId::anonymous(),
        minting_account_id: Some(minting_account.to_string()),
        last_purged_notification: Some(0),
    };

    pocket_ic.install_canister(
        cycle_minting_canister_id,
        cycle_minting_canister_wasm.into(),
        candid::encode_one(cycles_minting_canister_init_args).unwrap(),
        Some(super_admin),
    );

    let authorized_subnetwork_list_args = AuthorizedSubnetWorks {
        who: Some(platform_canister_id),
        subnets: application_subnets.clone(),
    };
    pocket_ic
        .update_call(
            cycle_minting_canister_id,
            CanisterId::anonymous(),
            "set_authorized_subnetwork_list",
            candid::encode_one(authorized_subnetwork_list_args).unwrap(),
        )
        .unwrap();

    let first_subnet_orchestrator_canister_id: Principal = pocket_ic
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
    let second_subnet_orchestrator_canister_id: Principal = pocket_ic
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

    //get alice canister-id
    let alice_principal_id = get_mock_user_alice_principal_id();
    let alice_canister_id: Principal = pocket_ic.update_call(first_subnet_orchestrator_canister_id, alice_principal_id, "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer", candid::encode_one(()).unwrap())
    .map(|res| {
        let canister_id: Principal = match res {
            PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("Canister call failed")
        };
        canister_id
    })
    .unwrap();

    let bob_principal_id = get_mock_user_bob_principal_id();
    let bob_canister_id: Principal = pocket_ic.update_call(second_subnet_orchestrator_canister_id, bob_principal_id, "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer", candid::encode_one(()).unwrap())
    .map(|res| {
        let canister_id: Principal = match res {
            PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("Canister call failed")
        };
        canister_id
    })
    .unwrap();

    // update controller id
    let mut mock_controller = MockConstantsWrapper::new();
    mock_controller
        .expect_get_hot_or_not_controller_id()
        .return_const(application_subnets[0].to_text());

    //update subnet known principal
    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "update_subnet_known_principal",
            candid::encode_args((
                first_subnet_orchestrator_canister_id,
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

    // transfer token
    let success = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "transfer_tokens_and_posts",
            candid::encode_args((bob_principal_id, bob_canister_id)).unwrap(),
        )
        .map(|reply_payload| {
            let success: Result<String, String> = match reply_payload {
                PocketICWasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 transfer_tokens_and_posts failed\n"),
            };
            success
        })
        .unwrap();

    assert_eq!(success, Err("Success".to_owned()));
}

// #[test]
fn error_when_receiver_is_already_migrated() {}

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
