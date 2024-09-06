pub mod types;

use core::{hash, time};
use std::{
    collections::{HashMap, HashSet}, fmt::Debug, fs, str::FromStr, time::SystemTime, vec
};
use ic_sns_init::pb::v1::{SnsInitPayload, FractionalDeveloperVotingPower, sns_init_payload::InitialTokenDistribution, DeveloperDistribution, SwapDistribution, NeuronDistribution, TreasuryDistribution, AirdropDistribution};
use ic_sns_swap::pb::v1::{NeuronBasketConstructionParameters};
use sha2::{Sha256, Digest};
use flate2::read::GzDecoder;
use std::io::Read;
use std::time::{Duration, UNIX_EPOCH};


use candid::{CandidType, Decode, Encode, Principal};
use icp_ledger;
use ic_cdk::api::{management_canister::provisional::CanisterSettings, time};
use ic_ledger_types::{AccountIdentifier, BlockIndex, Tokens, DEFAULT_SUBACCOUNT};
use pocket_ic::{PocketIc, PocketIcBuilder, WasmResult};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::{
        individual_user_template::{self, types::cdao::DeployedCdaoCanisters, types::error::CdaoDeployError},
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
    constant::{NNS_CYCLE_MINTING_CANISTER, NNS_LEDGER_CANISTER_ID, SNS_WASM_W_PRINCIPAL_ID, YRAL_POST_CACHE_CANISTER_ID},
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_global_super_admin_principal_id, get_mock_user_alice_principal_id, get_mock_user_charlie_principal_id, get_mock_user_dan_canister_id, v1::CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS
    },
};
use ic_base_types::PrincipalId;
use ic_sns_wasm::init::SnsWasmCanisterInitPayload;
pub type CanisterId = Principal;

pub const ICP_LEDGER_CANISTER_ID: &'static str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
pub const ICP_INDEX_CANISTER_ID: &'static str = "qhbym-qaaaa-aaaaa-aaafq-cai";

#[derive(CandidType, Deserialize, PartialEq, Eq, Hash, Serialize, Clone)]
struct Wasm {
    wasm: Vec<u8>,
    proposal_id: Option<u64>,
    canister_type: i32
}

impl Debug for Wasm {
    // dont print the wasm: Vec<u8> field
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wasm")
            .field("proposal_id", &self.proposal_id)
            .field("canister_type", &self.canister_type)
            .finish()
    }
}

#[derive(CandidType, Deserialize, PartialEq, Eq, Hash, Serialize, Clone, Debug)]
struct AddWasmPayload {
    hash: Vec<u8>,
    wasm: Option<Wasm>
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct AddWasmResultRecord {
    pub result: Option<ResultVariant>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum ResultVariant {
    Error(ErrorRecord),
    Hash(Vec<u8>),
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct ErrorRecord {
    pub message: String,
}

fn add_wasm(wasm_file: &[u8], canister_type: u32) -> AddWasmPayload {
    // Calculate the SHA-256 hash of the WASM file
    let mut hasher = Sha256::new();
    hasher.update(wasm_file);
    let file_hash = hasher.finalize();

    // Create a JSON object representing the WASM module
    let wasm_data = AddWasmPayload {
        // hash: hex::decode(hex::encode(file_hash)).unwrap(),
        hash: file_hash.to_vec(),
        // "wasm": base64::encode(wasm_file),
        // "canister_type": canister_type,
        wasm: Some(Wasm {
            wasm: wasm_file.to_vec(),
            proposal_id: None,
            canister_type: canister_type as i32
        })
    };

    ic_cdk::println!("Wasm data: {:?}\nType: {}, Hash: {}", wasm_data, canister_type, hex::encode(file_hash));

    wasm_data
}

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
    
    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "update_global_known_principal",
            candid::encode_args((KnownPrincipalType::CanisterIdSnsWasm, Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap())).unwrap(),
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

    pocket_ic.add_cycles(alice_cannister_id, 1_000_000_000_000_000);

    let offchain_account = get_mock_user_dan_canister_id();

    // let icp_ledger_canister_wasms = include_bytes!("../../../../../wasms/icp_ledger.wasm");
    let icp_ledger_canister_id = Principal::from_text(ICP_LEDGER_CANISTER_ID).unwrap();
    // let _ = pocket_ic.create_canister_with_id(
    //     Some(super_admin),
    //     None,
    //     Principal::from_text(ICP_LEDGER_CANISTER_ID).unwrap(),
    // );
    // let init_args = types::InitArgs {
    //     send_whitelist: vec![],
    //     token_symbol: Some("ICP".to_string()),
    //     transfer_fee: Some(types::Tokens { e8s: 10_000 }), // 10,000 e8s as the transfer fee
    //     minting_account: alice_cannister_id.to_string(),
    //     maximum_number_of_accounts: None,
    //     accounts_overflow_trim_quantity: None,
    //     transaction_window: None,
    //     max_message_size_bytes: None,
    //     icrc1_minting_account: None,
    //     archive_options: None,
    //     initial_values: vec![
    //         (
    //             minter_account.to_string(),
    //             types::Tokens { e8s: 100_000_000_000 }, // 100_000_000_000 e8s initial balance
    //         )
    //     ],
    //     token_name: Some("Local ICP".to_string()),
    //     feature_flags: None,
    // };
    // let icp_init_payload = types::LedgerCanisterPayload::Init(init_args);

    // let res = pocket_ic.reinstall_canister(
    //     icp_ledger_canister_id,
    //     icp_ledger_canister_wasms.to_vec(),
    //     Encode!(&icp_init_payload).unwrap(),
    //     Some(super_admin),
    // );
    // if !res.is_ok() {
    //     let err = res.unwrap_err();
    //     panic!("🛑 Error: {:?}", err);
    // }

    // let mint_icp_payload = types::CandidOperation::Mint { to: serde_bytes::ByteBuf::from(icp_ledger::AccountIdentifier::new(PrincipalId(offchain_account), None).to_address()), amount: types::Tokens { e8s: 100_000_000_000 } };
    // let res = pocket_ic.update_call(
    //     Principal::from_text(ICP_LEDGER_CANISTER_ID).unwrap(),
    //     alice_principal,
    //     "candid_mint",
    //     candid::encode_one(mint_icp_payload).unwrap(),
    // );

    let sns_wasm_w_canister_wasm = include_bytes!("../../../../../wasms/sns-wasm-canister.wasm");
    let sns_wasm_w_canister_id = Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap();

    let _ = pocket_ic.create_canister_with_id(
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

    // let sns_root_canister_wasm_gz = include_bytes!("../../../../../wasms/sns-root-canister.wasm.gz").to_vec();
    // let mut hasher = Sha256::new();
    // hasher.update(&sns_root_canister_wasm_gz);
    // let file_hash = hasher.finalize();
    // let file_hash_hex = hex::encode(file_hash);
    // let mut decoder = GzDecoder::new(&sns_root_canister_wasm_gz[..]);
    // let mut decompressed_data = Vec::new();
    // decoder.read_to_end(&mut decompressed_data).unwrap();
    // let file_hash_bytes = file_hash_hex.as_bytes().to_vec();

    // let mut hash_bytes = [0u8; 32];
    // hex::decode_to_slice("495e31370b14fa61c76bd1483c9f9ba66733793ee2963e8e44a231436a60bcc6", &mut hash_bytes).expect("Decoding failed");

    let res = pocket_ic.update_call(
        sns_wasm_w_canister_id, 
        super_admin, 
        "add_wasm", 
        candid::encode_one(add_wasm(include_bytes!("../../../../../wasms/root.wasm.gz"), 1)).unwrap()
    ).map(|res| {
        let response: AddWasmResultRecord = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    }).unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);
    
    let res = pocket_ic.update_call(
        sns_wasm_w_canister_id, 
        super_admin, 
        "add_wasm", 
        candid::encode_one(add_wasm(include_bytes!("../../../../../wasms/governance.wasm.gz"), 2)).unwrap()
    ).map(|res| {
        let response: AddWasmResultRecord = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    }).unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic.update_call(
        sns_wasm_w_canister_id, 
        super_admin, 
        "add_wasm", 
        candid::encode_one(add_wasm(include_bytes!("../../../../../wasms/ledger.wasm.gz"), 3)).unwrap()
    ).map(|res| {
        let response: AddWasmResultRecord = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    }).unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic.update_call(
        sns_wasm_w_canister_id, 
        super_admin, 
        "add_wasm", 
        candid::encode_one(add_wasm(include_bytes!("../../../../../wasms/swap.wasm.gz"), 4)).unwrap()
    ).map(|res| {
        let response: AddWasmResultRecord = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    }).unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic.update_call(
        sns_wasm_w_canister_id, 
        super_admin, 
        "add_wasm", 
        candid::encode_one(add_wasm(include_bytes!("../../../../../wasms/archive.wasm.gz"), 5)).unwrap()
    ).map(|res| {
        let response: AddWasmResultRecord = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    }).unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic.update_call(
        sns_wasm_w_canister_id, 
        super_admin, 
        "add_wasm", 
        candid::encode_one(add_wasm(include_bytes!("../../../../../wasms/index.wasm.gz"), 6)).unwrap()
    ).map(|res| {
        let response: AddWasmResultRecord = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    }).unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    for _ in 0..50 {
        pocket_ic.tick();
    }

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
    assert_eq!(res.len(), 6);
    let start = SystemTime::now();
   
    let sns_init_args = SnsInitPayload {
        confirmation_text: Some("Welcome to the jungle baby".to_string()),
        transaction_fee_e8s: Some(1u64),
        token_name: Some("Simulation Governance".to_string()),
        token_symbol: Some("SIMG".to_string()),
        proposal_reject_cost_e8s: Some(1u64),
        neuron_minimum_stake_e8s: Some(2u64),
        fallback_controller_principal_ids: vec![super_admin.to_string().clone()],
        logo: Some("data:image/png;base64,iVBORw0".to_string()),
        url: Some("https://google.com".to_string()),
        name: Some("Simulation Gov".to_string()),
        description: Some("Simulation gov desc".to_string()),
        neuron_minimum_dissolve_delay_to_vote_seconds: Some(1),
        initial_reward_rate_basis_points: Some(30u64),
        final_reward_rate_basis_points: Some(20u64),
        reward_rate_transition_duration_seconds: Some(1u64),
        max_dissolve_delay_seconds: Some(5u64),
        max_neuron_age_seconds_for_age_bonus: Some(1u64),
        max_dissolve_delay_bonus_percentage: Some(10u64),
        max_age_bonus_percentage: Some(10u64),
        initial_voting_period_seconds: Some(86401u64),
        wait_for_quiet_deadline_increase_seconds: Some(1u64),
        restricted_countries: None,
        dapp_canisters: None,
        min_participants: Some(1),
        min_icp_e8s: None,
        max_icp_e8s: None,
        min_direct_participation_icp_e8s: Some(15u64),
        min_participant_icp_e8s: Some(2000u64),
        max_direct_participation_icp_e8s: Some(100_000_000u64),
        max_participant_icp_e8s: Some(100_000_000u64),
        swap_start_timestamp_seconds: Some(start.duration_since(UNIX_EPOCH).unwrap().as_secs()),
        swap_due_timestamp_seconds: Some(start.duration_since(UNIX_EPOCH).unwrap().as_secs() + 300), // year 3000 - hopefully we'll all be gone by then,
        neuron_basket_construction_parameters: Some(NeuronBasketConstructionParameters {
            count: 2,
            dissolve_delay_interval_seconds: 2
        }),
        nns_proposal_id: Some(1),
        neurons_fund_participation: Some(false),
        neurons_fund_participants: None,
        token_logo: Some("data:image/png;base64,iVBORw0".to_string()),
        neurons_fund_participation_constraints: None,
        initial_token_distribution: Some(
            InitialTokenDistribution::FractionalDeveloperVotingPower(
                FractionalDeveloperVotingPower {
                    airdrop_distribution: Some(AirdropDistribution {
                        airdrop_neurons: vec![]
                    }),
                    developer_distribution: Some(DeveloperDistribution{
                        developer_neurons: vec![
                            NeuronDistribution {
                                controller: Some(PrincipalId::from_str(&alice_principal.to_string()).unwrap()),
                                stake_e8s: 4_500_000,
                                memo: 0,
                                dissolve_delay_seconds: 2,
                                vesting_period_seconds: None
                            }
                        ]
                    }),
                    treasury_distribution: Some(TreasuryDistribution {
                        total_e8s: 10_000_000
                    }),
                    swap_distribution: Some(SwapDistribution {
                        total_e8s: 5_000_000,
                        initial_swap_amount_e8s: 5_000_000,
                    }),
                }
            )
        ),
    };

    let res = pocket_ic.update_call(
        alice_cannister_id,
        alice_principal,
        "deploy_cdao_sns",
        candid::encode_args((sns_init_args, 600 as u64)).unwrap(),
    ).map(|res| {
        let response: Result<DeployedCdaoCanisters, CdaoDeployError> = match res {
            WasmResult::Reply(payload) => {
                ic_cdk::println!("🧪 Call made");
                Decode!(&payload, Result<DeployedCdaoCanisters, CdaoDeployError>).unwrap()
            },
            _ => panic!("\n🛑 deploy cdao failed with {:?}", res)
        };
        response
    }).unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic.query_call(
        alice_cannister_id,
        alice_principal,
        "get_well_known_principal_value", 
        candid::encode_one((KnownPrincipalType::CanisterIdSnsWasm)).unwrap(),
    ).map(|res| {
        let response: Option<Principal> = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    }).unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res.unwrap().to_string());

    let res = pocket_ic.query_call(
        alice_cannister_id,
        alice_principal,
        "deployed_cdao_canisters", 
        candid::encode_one(()).unwrap(),
    ).map(|res| {
        let response: Vec<DeployedCdaoCanisters> = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    }).unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);
}
