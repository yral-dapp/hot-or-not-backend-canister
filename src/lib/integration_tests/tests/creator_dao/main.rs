pub mod test_custom_governance_upgrade;
pub mod test_deletion_of_creator_tokens;
pub mod test_number_of_creator_tokens;
pub mod types;
pub mod utils;

use ic_ledger_types::Memo;
use ic_sns_governance::pb::v1::governance::Version;
use ic_sns_governance::pb::v1::{
    manage_neuron, neuron, Account, GetRunningSnsVersionRequest, GetRunningSnsVersionResponse,
    ListNeurons, ListNeuronsResponse, ManageNeuron, ManageNeuronResponse,
};
use ic_sns_init::pb::v1::{
    sns_init_payload::InitialTokenDistribution, AirdropDistribution, DeveloperDistribution,
    FractionalDeveloperVotingPower, NeuronDistribution, SnsInitPayload, SwapDistribution,
    TreasuryDistribution,
};
use ic_sns_swap::pb::v1::{
    GetInitRequest, GetInitResponse, NeuronBasketConstructionParameters, NewSaleTicketRequest,
    NewSaleTicketResponse, RefreshBuyerTokensRequest, RefreshBuyerTokensResponse,
};
use sha2::{Digest, Sha256};
use shared_utils::canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs;
use shared_utils::canister_specific::individual_user_template::types::error::AirdropError;
use shared_utils::constant::{
    SNS_TOKEN_ARCHIVE_MODULE_HASH, SNS_TOKEN_GOVERNANCE_MODULE_HASH, SNS_TOKEN_INDEX_MODULE_HASH,
    SNS_TOKEN_LEDGER_MODULE_HASH, SNS_TOKEN_ROOT_MODULE_HASH, SNS_TOKEN_SWAP_MODULE_HASH,
};
use shared_utils::types::creator_dao_stats::CreatorDaoTokenStats;
use test_utils::setup::env::pocket_ic_env::provision_subnet_orchestrator_canister;
use std::time::{Duration, UNIX_EPOCH};
use std::{collections::HashMap, fmt::Debug, str::FromStr, time::SystemTime, vec};
use test_utils::setup::test_constants::get_mock_user_bob_principal_id;
use utils::{setup_default_sns_creator_token, setup_sns_w_canister_for_creator_dao};

use candid::{encode_args, CandidType, Decode, Encode, Nat, Principal};
use ic_base_types::PrincipalId;
use ic_sns_wasm::init::SnsWasmCanisterInitPayload;
use icp_ledger::Subaccount;
use pocket_ic::WasmResult;
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::{
        types::cdao::DeployedCdaoCanisters, types::error::CdaoDeployError,
    },
    common::types::known_principal::KnownPrincipalType,
    constant::SNS_WASM_W_PRINCIPAL_ID,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
        get_mock_user_charlie_principal_id,
    },
};

pub const ICP_LEDGER_CANISTER_ID: &'static str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
pub const ICP_INDEX_CANISTER_ID: &'static str = "qhbym-qaaaa-aaaaa-aaafq-cai";

#[derive(CandidType, Deserialize, PartialEq, Eq, Hash, Serialize, Clone)]
struct Wasm {
    wasm: Vec<u8>,
    proposal_id: Option<u64>,
    canister_type: i32,
}

impl Debug for Wasm {
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
    wasm: Option<Wasm>,
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
    let mut hasher = Sha256::new();
    hasher.update(wasm_file);
    let file_hash = hasher.finalize();

    let wasm_data = AddWasmPayload {
        hash: file_hash.to_vec(),
        wasm: Some(Wasm {
            wasm: wasm_file.to_vec(),
            proposal_id: None,
            canister_type: canister_type as i32,
        }),
    };

    ic_cdk::println!(
        "Wasm data: {:?}\nType: {}, Hash: {}",
        wasm_data,
        canister_type,
        hex::encode(file_hash)
    );

    wasm_data
}

#[test]
fn creator_dao_tests() {
    let (pocket_ic, mut known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = get_global_super_admin_principal_id();

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
            candid::encode_args((
                KnownPrincipalType::CanisterIdSnsWasm,
                Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap(),
            ))
            .unwrap(),
        )
        .unwrap();

    let subnet_orchestrator_canister_id = provision_subnet_orchestrator_canister(
        &pocket_ic,
        &known_principal,
        1,
        Some(charlie_global_admin),
    );

    let alice_principal = get_mock_user_alice_principal_id();
    let alice_canister_id: Principal = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id,
            alice_principal,
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

    let alice_initial_cycle_balance = pocket_ic.cycle_balance(alice_canister_id);
    let sns_wasm_w_canister_id = Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap();

    setup_sns_w_canister_for_creator_dao(&pocket_ic, super_admin);

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

    let deployed_canister = setup_default_sns_creator_token(
        &pocket_ic,
        super_admin,
        alice_principal,
        alice_canister_id,
    );

    let ledger_canister = deployed_canister.ledger;
    let root_canister = deployed_canister.root;
    let gov_canister = deployed_canister.governance;

    let res = pocket_ic
        .query_call(
            ledger_canister,
            alice_principal,
            "icrc1_balance_of",
            candid::encode_one(types::Icrc1BalanceOfArg {
                owner: alice_principal,
                subaccount: None,
            })
            .unwrap(),
        )
        .map(|res| {
            let response = match res {
                WasmResult::Reply(payload) => Decode!(&payload, Nat).unwrap(),
                _ => panic!("\n🛑 icrc1_balance_of failed \n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 SNS token Balance of alice: {:?}", res);

    let tx_fee = 1u64;
    let expected_balance = Nat::from(60_000_000_000 - tx_fee);
    ic_cdk::println!("🧪 Expected Balance: {:?}", expected_balance);

    let alice_canister_final_cycle_balance = pocket_ic.cycle_balance(alice_canister_id);

    assert!(alice_canister_final_cycle_balance > alice_initial_cycle_balance);

    assert!(res == expected_balance);

    let sns_running_version = pocket_ic
        .query_call(
            gov_canister,
            Principal::anonymous(),
            "get_running_sns_version",
            candid::encode_one(GetRunningSnsVersionRequest {}).unwrap(),
        )
        .map(|wasm_result| {
            let result: GetRunningSnsVersionResponse = match wasm_result {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Call to get version of sns failed"),
            };
            result
        })
        .unwrap();

    let deployed_sns_version = Version {
        governance_wasm_hash: hex::decode(SNS_TOKEN_GOVERNANCE_MODULE_HASH).unwrap(),
        root_wasm_hash: hex::decode(SNS_TOKEN_ROOT_MODULE_HASH).unwrap(),
        ledger_wasm_hash: hex::decode(SNS_TOKEN_LEDGER_MODULE_HASH).unwrap(),
        swap_wasm_hash: hex::decode(SNS_TOKEN_SWAP_MODULE_HASH).unwrap(),
        index_wasm_hash: hex::decode(SNS_TOKEN_INDEX_MODULE_HASH).unwrap(),
        archive_wasm_hash: hex::decode(SNS_TOKEN_ARCHIVE_MODULE_HASH).unwrap(),
    };

    assert_eq!(
        sns_running_version.deployed_version,
        Some(deployed_sns_version)
    );
    //Upgrade Governance Canister and check the running version
    let bob = get_mock_user_bob_principal_id();
    let bob_canister_id: Principal = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id,
            bob,
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

    // simulating off-chain allocation (kinda)
    let decimals = pocket_ic
        .query_call(
            ledger_canister,
            alice_canister_id,
            "icrc1_decimals",
            Encode!(&()).unwrap(),
        )
        .map(|res| {
            let response: u8 = match res {
                WasmResult::Reply(payload) => Decode!(&payload, u8).unwrap(),
                _ => panic!("\n🛑 icrc1_transfer failed with: {:?}", res),
            };
            response
        })
        .unwrap();

    let transfer_args = types::TransferArg {
        from_subaccount: None,
        to: types::Account {
            owner: alice_canister_id,
            subaccount: None,
        },
        fee: None,
        created_at_time: None,
        memo: None,
        amount: Nat::from(200u32) * 10u64.pow(decimals.into()),
    };
    let transfer = pocket_ic
        .update_call(
            ledger_canister,
            alice_principal,
            "icrc1_transfer",
            Encode!(&transfer_args).unwrap(),
        )
        .map(|res| {
            let response: types::TransferResult = match res {
                WasmResult::Reply(payload) => Decode!(&payload, types::TransferResult).unwrap(),
                _ => panic!("\n🛑 icrc1_transfer failed with: {:?}", res),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", transfer);

    // claiming airdrop
    let res = pocket_ic
        .update_call(
            alice_canister_id,
            bob,
            "request_airdrop",
            encode_args((
                root_canister,
                None::<Memo>,
                Nat::from(100u64) * 10u64.pow(decimals.into()),
                bob_canister_id,
            ))
            .unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<(), AirdropError> = match reply_payload {
                WasmResult::Reply(payload) => Decode!(&payload, Result<(), AirdropError>).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        });
    ic_cdk::println!("🧪 Result: {:?}", res);
    assert!(res.as_ref().unwrap().is_ok());

    // trying to claim the airdrop again
    let res: Result<Result<(), AirdropError>, pocket_ic::UserError> = pocket_ic
        .update_call(
            alice_canister_id,
            bob,
            "request_airdrop",
            encode_args((
                root_canister,
                None::<Memo>,
                Nat::from(100u64) * 10u64.pow(decimals.into()),
                bob_canister_id,
            ))
            .unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<(), AirdropError> = match reply_payload {
                WasmResult::Reply(payload) => Decode!(&payload, Result<(), AirdropError>).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        });

    ic_cdk::println!("🧪 Result: {:?}", res);
    assert!(
        res.as_ref().unwrap().is_err() && res.unwrap() == Err(AirdropError::AlreadyClaimedAirdrop)
    );

    // trying to claim the airdrop with the wrong canister id
    let res: Result<Result<(), AirdropError>, pocket_ic::UserError> = pocket_ic
        .update_call(
            alice_canister_id,
            bob,
            "request_airdrop",
            encode_args((
                root_canister,
                None::<Memo>,
                Nat::from(100u64) * 10u64.pow(decimals.into()),
                Principal::anonymous(),
            ))
            .unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<(), AirdropError> = match reply_payload {
                WasmResult::Reply(payload) => Decode!(&payload, Result<(), AirdropError>).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        });

    ic_cdk::println!("🧪 Result: {:?}", res);
    assert!(res.unwrap().is_err());

    let deployed_cdao = pocket_ic
        .query_call(
            alice_canister_id,
            alice_principal,
            "deployed_cdao_canisters",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let response: Vec<DeployedCdaoCanisters> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", deployed_cdao);
    assert!(deployed_cdao[0]
        .airdrop_info
        .is_airdrop_claimed(&bob)
        .unwrap());

    let bob_bal = pocket_ic
        .query_call(
            ledger_canister,
            alice_canister_id,
            "icrc1_balance_of",
            candid::encode_one(types::Icrc1BalanceOfArg {
                owner: bob,
                subaccount: None,
            })
            .unwrap(),
        )
        .map(|res| match res {
            WasmResult::Reply(payload) => Decode!(&payload, Nat).unwrap(),
            _ => panic!("\n🛑 get bob principal bal failed\n"),
        })
        .unwrap();
    ic_cdk::println!("🧪 SNS token Balance of bob principal: {:?}", bob_bal);

    let alice_bal = pocket_ic
        .query_call(
            ledger_canister,
            alice_canister_id,
            "icrc1_balance_of",
            candid::encode_one(types::Icrc1BalanceOfArg {
                owner: alice_canister_id,
                subaccount: None,
            })
            .unwrap(),
        )
        .map(|res| match res {
            WasmResult::Reply(payload) => Decode!(&payload, Nat).unwrap(),
            _ => panic!("\n🛑 get alice canister bal failed\n"),
        })
        .unwrap();
    ic_cdk::println!("🧪 SNS token Balance of alice canister: {:?}", alice_bal);

    assert!(bob_bal == Nat::from(100u64) * 10u64.pow(decimals.into()));

    for _ in 0..5 {
        pocket_ic.tick();
    }

    let creator_dao_stats = pocket_ic
        .query_call(
            platform_canister_id,
            charlie_global_admin,
            "get_creator_dao_stats",
            candid::encode_one(()).unwrap(),
        )
        .map(|wasm_result| {
            let result: CreatorDaoTokenStats = match wasm_result {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(e) => panic!("\n failed to get creator dao stats {}", e),
            };
            result
        })
        .unwrap();

    assert_eq!(creator_dao_stats.total_number_of_creator_dao_tokens, 1);

    pocket_ic
        .update_call(
            platform_canister_id,
            charlie_global_admin,
            "collect_creator_dao_stats_in_the_network",
            candid::encode_one(()).unwrap(),
        )
        .unwrap();

    for _ in 0..5 {
        pocket_ic.tick();
    }

    let creator_dao_stats = pocket_ic
        .query_call(
            platform_canister_id,
            charlie_global_admin,
            "get_creator_dao_stats",
            candid::encode_one(()).unwrap(),
        )
        .map(|wasm_result| {
            let result: CreatorDaoTokenStats = match wasm_result {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(e) => panic!("\n failed to get creator dao stats {}", e),
            };
            result
        })
        .unwrap();

    assert_eq!(creator_dao_stats.total_number_of_creator_dao_tokens, 1);
}
