use std::{
    collections::HashMap,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use candid::{Decode, Principal};
use ic_base_types::PrincipalId;
use ic_sns_init::pb::v1::{
    sns_init_payload::InitialTokenDistribution, AirdropDistribution, DeveloperDistribution,
    FractionalDeveloperVotingPower, NeuronDistribution, SnsInitPayload, SwapDistribution,
    TreasuryDistribution,
};
use ic_sns_swap::pb::v1::NeuronBasketConstructionParameters;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cdao::DeployedCdaoCanisters, error::CdaoDeployError,
    },
    common::types::known_principal::KnownPrincipalType,
    constant::{MAX_LIMIT_FOR_CREATOR_DAO_SNS_TOKEN, SNS_WASM_W_PRINCIPAL_ID},
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
        get_mock_user_charlie_principal_id,
    },
};

use crate::utils::setup_sns_w_canister_for_creator_dao;

#[test]
pub fn test_number_of_creator_tokens() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = get_global_super_admin_principal_id();

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
            candid::encode_args((
                KnownPrincipalType::CanisterIdSnsWasm,
                Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap(),
            ))
            .unwrap(),
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

    for i in 0..150 {
        pocket_ic.tick();
    }

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
    let start = SystemTime::now();

    let tx_fee = 1u64;

    let sns_init_args = SnsInitPayload {
        confirmation_text: Some("GET RICH QUICK".to_string()),
        transaction_fee_e8s: Some(tx_fee),
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
            dissolve_delay_interval_seconds: 2,
        }),
        nns_proposal_id: Some(1),
        neurons_fund_participation: Some(false),
        token_logo: Some("data:image/png;base64,iVBORw0".to_string()),
        neurons_fund_participation_constraints: None,
        initial_token_distribution: Some(InitialTokenDistribution::FractionalDeveloperVotingPower(
            FractionalDeveloperVotingPower {
                airdrop_distribution: Some(AirdropDistribution {
                    airdrop_neurons: vec![],
                }),
                developer_distribution: Some(DeveloperDistribution {
                    developer_neurons: vec![
                        NeuronDistribution {
                            controller: Some(
                                PrincipalId::from_str(&alice_principal.to_string()).unwrap(),
                            ),
                            stake_e8s: 60_000_000_000,
                            memo: 0,
                            dissolve_delay_seconds: 0,
                            vesting_period_seconds: None,
                        },
                        NeuronDistribution {
                            controller: Some(
                                PrincipalId::from_str(&alice_principal.to_string()).unwrap(),
                            ),
                            stake_e8s: 100_000,
                            memo: 1,
                            dissolve_delay_seconds: 2,
                            vesting_period_seconds: None,
                        },
                    ],
                }),
                treasury_distribution: Some(TreasuryDistribution {
                    total_e8s: 10_000_000,
                }),
                swap_distribution: Some(SwapDistribution {
                    total_e8s: 65_000_000_000,
                    initial_swap_amount_e8s: 5_000_000,
                }),
            },
        )),
    };

    for _ in 0..MAX_LIMIT_FOR_CREATOR_DAO_SNS_TOKEN {
        let creator_dao_deployed_cans_result = pocket_ic
            .update_call(
                alice_canister_id,
                alice_principal,
                "deploy_cdao_sns",
                candid::encode_args((sns_init_args.clone(), 300 as u64)).unwrap(),
            )
            .map(|res| {
                let response: Result<DeployedCdaoCanisters, CdaoDeployError> = match res {
                    WasmResult::Reply(payload) => {
                        ic_cdk::println!("🧪 Call made");
                        Decode!(&payload, Result<DeployedCdaoCanisters, CdaoDeployError>).unwrap()
                    }
                    _ => panic!("\n🛑 deploy cdao failed with {:?}", res),
                };
                response
            })
            .unwrap();

        assert!(creator_dao_deployed_cans_result.is_ok());
    }
    let creator_dao_deployed_cans_result = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal,
            "deploy_cdao_sns",
            candid::encode_args((sns_init_args, 300 as u64)).unwrap(),
        )
        .map(|res| {
            let response: Result<DeployedCdaoCanisters, CdaoDeployError> = match res {
                WasmResult::Reply(payload) => {
                    ic_cdk::println!("🧪 Call made");
                    Decode!(&payload, Result<DeployedCdaoCanisters, CdaoDeployError>).unwrap()
                }
                _ => panic!("\n🛑 deploy cdao failed with {:?}", res),
            };
            response
        })
        .unwrap();

    assert!(creator_dao_deployed_cans_result.is_err());
    assert!(matches!(
        creator_dao_deployed_cans_result, Err(e) if e == CdaoDeployError::TokenLimit(MAX_LIMIT_FOR_CREATOR_DAO_SNS_TOKEN)))
}
