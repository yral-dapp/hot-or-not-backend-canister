use ic_sns_governance::pb::v1::{
    manage_neuron, neuron, Account,
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
use shared_utils::canister_specific::individual_user_template::types::cdao::{SwapTokenData, TokenPairs};
use shared_utils::canister_specific::individual_user_template::types::error::{AirdropError, SwapError};
use test_utils::setup::test_constants::get_mock_user_bob_principal_id;
use shared_utils::constant::GLOBAL_SUPER_ADMIN_USER_ID_V1;
use crate::types::{ApproveArgs, ApproveError};
use crate::ICP_LEDGER_CANISTER_ID;
use std::time::{Duration, UNIX_EPOCH};
use std::{str::FromStr, time::SystemTime, vec};
use crate::utils::{setup_default_sns_creator_token, setup_sns_w_canister_for_creator_dao};

use candid::{Decode, Encode, Nat, Principal};
use ic_base_types::PrincipalId;
use icp_ledger::Subaccount;
use pocket_ic::WasmResult;
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

#[test]
fn test_swap_flow() {
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

    for _ in 0..50 {
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

    setup_sns_w_canister_for_creator_dao(&pocket_ic, super_admin);

    let deployed_canister = setup_default_sns_creator_token(
        &pocket_ic,
        super_admin,
        alice_principal,
        alice_canister_id,
    );

    let ledger_canister = deployed_canister.ledger;

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
    // Swap Tests
    // initing another cdao token for bob

    let bob_initial_cycle_balance = pocket_ic.cycle_balance(bob_canister_id);

    let start = SystemTime::now();

    let tx_fee = 1u64;

    let sns_init_args = SnsInitPayload {
        confirmation_text: Some("GET RICH QUICK2".to_string()),
        transaction_fee_e8s: Some(tx_fee),
        token_name: Some("Simulation Governance2".to_string()),
        token_symbol: Some("SIMG2".to_string()),
        proposal_reject_cost_e8s: Some(1u64),
        neuron_minimum_stake_e8s: Some(2u64),
        fallback_controller_principal_ids: vec![super_admin.to_string().clone()],
        logo: Some("data:image/png;base64,iVBORw0".to_string()),
        url: Some("https://google.com2".to_string()),
        name: Some("Simulation Gov2".to_string()),
        description: Some("Simulation gov desc2".to_string()),
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
                                PrincipalId::from_str(&bob.to_string()).unwrap(),
                            ),
                            stake_e8s: 60_000_000_000,
                            memo: 0,
                            dissolve_delay_seconds: 0,
                            vesting_period_seconds: None,
                        },
                        NeuronDistribution {
                            controller: Some(
                                PrincipalId::from_str(&bob.to_string()).unwrap(),
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


    let res = pocket_ic
        .update_call(
            bob_canister_id,
            bob,
            "deploy_cdao_sns",
            candid::encode_args((sns_init_args, 300u64)).unwrap(),
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
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .query_call(
            bob_canister_id,
            bob,
            "get_well_known_principal_value",
            candid::encode_one((KnownPrincipalType::CanisterIdSnsWasm)).unwrap(),
        )
        .map(|res| {
            let response: Option<Principal> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_well_known_principal_value failed"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res.unwrap().to_string());

    let res = pocket_ic
        .query_call(
            bob_canister_id,
            bob,
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
    ic_cdk::println!("🧪 Result: {:?}", res);
    for can in &res {
        ic_cdk::println!("🧪 Gov Canister ID: {:?}", can.governance.to_string());
        ic_cdk::println!("🧪 Ind Canister ID: {:?}", can.index.to_string());
        ic_cdk::println!("🧪 Ldg Canister ID: {:?}", can.ledger.to_string());
        ic_cdk::println!("🧪 Rrt Canister ID: {:?}", can.root.to_string());
        ic_cdk::println!("🧪 Swp Canister ID: {:?}", can.swap.to_string());
    }

    assert!(res.len() == 1);
    let res = res[0].clone();
    let swap_canister = res.swap;
    let gov_canister = res.governance;
    let bob_ledger_canister = res.ledger;

    ic_cdk::println!("🧪🧪🧪 Swap Canister ID: {:?}", swap_canister.to_string());

    let res = pocket_ic
        .query_call(
            Principal::from_text(ICP_LEDGER_CANISTER_ID).unwrap(),
            super_admin,
            "icrc1_total_supply",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let response = match res {
                WasmResult::Reply(payload) => Decode!(&payload, Nat).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    // check super admin icp balance
    let res = pocket_ic
        .query_call(
            Principal::from_text(ICP_LEDGER_CANISTER_ID).unwrap(),
            super_admin,
            "icrc1_balance_of",
            candid::encode_one(crate::types::Icrc1BalanceOfArg {
                owner: super_admin,
                subaccount: None,
            })
            .unwrap(),
        )
        .map(|res| {
            let response = match res {
                WasmResult::Reply(payload) => Decode!(&payload, Nat).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);
    
    pocket_ic.advance_time(Duration::from_secs(200));
    pocket_ic.tick();

    let res = pocket_ic
        .update_call(
            swap_canister,
            super_admin,
            "new_sale_ticket",
            candid::encode_one(NewSaleTicketRequest {
                amount_icp_e8s: 1000000,
                subaccount: None,
            })
            .unwrap(),
        )
        .map(|res| {
            let response: NewSaleTicketResponse = match res {
                WasmResult::Reply(payload) => Decode!(&payload, NewSaleTicketResponse).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let subaccount = Subaccount::from(&PrincipalId(super_admin));
    let transfer_args = crate::types::Transaction {
        memo: Some(vec![0]),
        amount: Nat::from(1000000 as u64),
        fee: Some(Nat::from(0 as u64)),
        from_subaccount: None,
        to: crate::types::Recipient {
            owner: swap_canister,
            subaccount: Some(subaccount.to_vec()),
        },
        created_at_time: None,
    };
    let res = pocket_ic
        .update_call(
            Principal::from_text(ICP_LEDGER_CANISTER_ID).unwrap(),
            super_admin,
            "icrc1_transfer",
            Encode!(&transfer_args).unwrap(),
        )
        .map(|res| {
            let response: crate::types::TransferResult = match res {
                WasmResult::Reply(payload) => Decode!(&payload, crate::types::TransferResult).unwrap(),
                _ => panic!("\n🛑 icrc1_transfer failed with: {:?}", res),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            swap_canister,
            super_admin,
            "refresh_buyer_tokens",
            candid::encode_one(RefreshBuyerTokensRequest {
                buyer: super_admin.to_string(),
                confirmation_text: Some("GET RICH QUICK2".to_string()),
            })
            .unwrap(),
        )
        .map(|res| {
            let response: RefreshBuyerTokensResponse = match res {
                WasmResult::Reply(payload) => {
                    Decode!(&payload, RefreshBuyerTokensResponse).unwrap()
                }
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    pocket_ic.advance_time(Duration::from_secs(301));
    for _ in 0..500 {
        pocket_ic.tick();
    }

    let res = pocket_ic
        .query_call(
            swap_canister,
            super_admin,
            "get_init",
            candid::encode_one(GetInitRequest {}).unwrap(),
        )
        .map(|res| {
            let response = match res {
                WasmResult::Reply(payload) => Decode!(&payload, GetInitResponse).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            gov_canister,
            super_admin,
            "list_neurons",
            candid::encode_one(ListNeurons {
                of_principal: Some(PrincipalId(bob)),
                limit: 2,
                start_page_at: None,
            })
            .unwrap(),
        )
        .map(|res| {
            let response: ListNeuronsResponse = match res {
                WasmResult::Reply(payload) => Decode!(&payload, ListNeuronsResponse).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let neurons = res.neurons;
    let mut ix = 0;
    if neurons[1].dissolve_state.is_some() {
        if let Some(neuron::DissolveState::DissolveDelaySeconds(x)) =
            neurons[1].dissolve_state.as_ref()
        {
            if *x == 0 {
                ix = 1;
            }
        }
    }
    let neuron_id = neurons[ix].id.as_ref().unwrap().id.clone();
    let amount = neurons[ix].cached_neuron_stake_e8s;
    let manage_neuron_arg = ManageNeuron {
        subaccount: neuron_id,
        command: Some(manage_neuron::Command::Disburse(manage_neuron::Disburse {
            to_account: Some(Account {
                owner: Some(PrincipalId(bob)),
                subaccount: None,
            }),
            amount: Some(manage_neuron::disburse::Amount { e8s: amount }),
        })),
    };
    
    pocket_ic.advance_time(Duration::from_secs(250));
    for _ in 0..10 {
        pocket_ic.tick();
    }

    let res = pocket_ic
        .update_call(
            gov_canister,
            bob,
            "manage_neuron",
            candid::encode_one(manage_neuron_arg).unwrap(),
        )
        .map(|res| {
            let response: ManageNeuronResponse = match res {
                WasmResult::Reply(payload) => Decode!(&payload, ManageNeuronResponse).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .query_call(
            bob_ledger_canister,
            bob,
            "icrc1_balance_of",
            candid::encode_one(crate::types::Icrc1BalanceOfArg {
                owner: bob,
                subaccount: None,
            })
            .unwrap(),
        )
        .map(|res| {
            let response = match res {
                WasmResult::Reply(payload) => Decode!(&payload, Nat).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 SNS token Balance of bob: {:?}", res);

    let expected_balance = Nat::from(60_000_000_000 - tx_fee);
    ic_cdk::println!("🧪 Expected Balance: {:?}", expected_balance);

    let bob_canister_final_cycle_balance = pocket_ic.cycle_balance(bob_canister_id);

    assert!(bob_canister_final_cycle_balance > bob_initial_cycle_balance);

    assert!(res == expected_balance);

    let res = pocket_ic.update_call(ledger_canister, alice_principal, "icrc2_approve", 
    candid::encode_one(ApproveArgs{
        from_subaccount: None,
        spender: crate::types::Account {
            owner: bob_canister_id,
            subaccount: None
        },
        amount: Nat::from(100u32) * Nat::from(10u32.pow(8)),
        expected_allowance: None,
        memo: None,
        expires_at: None,
        fee: None,
        created_at_time: None
    }, ).unwrap())
    .map(|res| {
        let response = match res {
            WasmResult::Reply(payload) => Decode!(&payload, Result<Nat, ApproveError>).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    })
    .unwrap();
    ic_cdk::println!("🧪 Swap Request from alice principal to bob canister: {:?}", res);
    assert!(res.is_ok());

    let res = pocket_ic.update_call(bob_ledger_canister, bob, "icrc2_approve", 
    candid::encode_one(ApproveArgs{
        from_subaccount: None,
        spender: crate::types::Account {
            owner: bob_canister_id,
            subaccount: None
        },
        amount: Nat::from(100u32) * Nat::from(10u32.pow(8)),
        expected_allowance: None,
        memo: None,
        expires_at: None,
        fee: None,
        created_at_time: None
    }, ).unwrap())
    .map(|res| {
        let response = match res {
            WasmResult::Reply(payload) => Decode!(&payload, Result<Nat, ApproveError>).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    })
    .unwrap();
    ic_cdk::println!("🧪 Swap Request from alice principal to bob canister: {:?}", res);
    assert!(res.is_ok());


    let res = pocket_ic
    .update_call(
        bob_canister_id,
        bob,
        "swap_request_action",
        candid::Encode!(
            &TokenPairs{
                token_a: SwapTokenData{
                    ledger: ledger_canister,
                    amt: 100u32.into()
                },
                token_b: SwapTokenData{
                    ledger: bob_ledger_canister,
                    amt: 10u32.into()
                }
            }, 
            &alice_principal
        ).unwrap()
    )
    .map(|res| {
        let response = match res {
            WasmResult::Reply(payload) => Decode!(&payload, Result<(), SwapError>).unwrap(),
            _ => panic!("\n🛑 get requester principals canister id failed\n"),
        };
        response
    })
    .unwrap();
    ic_cdk::println!("🧪 Accepting the swap from bob principal to bob canister: {:?}", res);
    assert!(res.is_ok());

    let res = pocket_ic.update_call(bob_canister_id, Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID_V1).unwrap(), "update_last_swap_price", candid::encode_args((bob_ledger_canister, 100f64)).unwrap()).unwrap();
    ic_cdk::println!("🧪 Updating the last swap price: {:?}", res);

    let deployed_cdao = pocket_ic
    .query_call(
        bob_canister_id,
        bob,
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

    assert!(deployed_cdao[0].last_swapped_price == Some(100f64), "Price setting failed!");
}
