use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use candid::{Decode, Encode, Nat, Principal};
use ic_base_types::PrincipalId;
use ic_sns_governance::pb::v1::{
    manage_neuron, neuron, Account, ListNeurons, ListNeuronsResponse, ManageNeuron,
    ManageNeuronResponse,
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
use ic_sns_wasm::init::SnsWasmCanisterInitPayload;
use icp_ledger::Subaccount;
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cdao::DeployedCdaoCanisters, error::CdaoDeployError,
    },
    common::types::known_principal::KnownPrincipalType,
    constant::SNS_WASM_W_PRINCIPAL_ID,
};

use crate::{add_wasm, types, AddWasmResultRecord, ICP_LEDGER_CANISTER_ID};

pub fn setup_sns_w_canister_for_creator_dao(pocket_ic: &PocketIc, super_admin: Principal) {
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

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/root.wasm.gz"),
                1,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/governance.wasm.gz"),
                2,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/ledger.wasm.gz"),
                3,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/swap.wasm.gz"),
                4,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/archive.wasm.gz"),
                5,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    let res = pocket_ic
        .update_call(
            sns_wasm_w_canister_id,
            super_admin,
            "add_wasm",
            candid::encode_one(add_wasm(
                include_bytes!("../../../../../wasms/index.wasm.gz"),
                6,
            ))
            .unwrap(),
        )
        .map(|res| {
            let response: AddWasmResultRecord = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    for _ in 0..50 {
        pocket_ic.tick();
    }
}

pub fn setup_default_sns_creator_token(
    pocket_ic: &PocketIc,
    super_admin: Principal,
    user_principal: Principal,
    user_canister_id: Principal,
) -> DeployedCdaoCanisters {
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
                                PrincipalId::from_str(&user_principal.to_string()).unwrap(),
                            ),
                            stake_e8s: 60_000_000_000,
                            memo: 0,
                            dissolve_delay_seconds: 0,
                            vesting_period_seconds: None,
                        },
                        NeuronDistribution {
                            controller: Some(
                                PrincipalId::from_str(&user_principal.to_string()).unwrap(),
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

    let creator_cdao_deploy_cans_result = pocket_ic
        .update_call(
            user_canister_id,
            user_principal,
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

    assert!(creator_cdao_deploy_cans_result.is_ok());

    let deployed_canisters = pocket_ic
        .query_call(
            user_canister_id,
            user_principal,
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
    ic_cdk::println!("🧪 Result: {:?}", deployed_canisters);
    for can in &deployed_canisters {
        ic_cdk::println!("🧪 Gov Canister ID: {:?}", can.governance.to_string());
        ic_cdk::println!("🧪 Ind Canister ID: {:?}", can.index.to_string());
        ic_cdk::println!("🧪 Ldg Canister ID: {:?}", can.ledger.to_string());
        ic_cdk::println!("🧪 Rrt Canister ID: {:?}", can.root.to_string());
        ic_cdk::println!("🧪 Swp Canister ID: {:?}", can.swap.to_string());
    }

    assert!(deployed_canisters.len() == 1);
    let res = deployed_canisters[0].clone();
    let root_canister = res.root;
    let swap_canister = res.swap;
    let gov_canister = res.governance;
    let ledger_canister = res.ledger;

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
            candid::encode_one(types::Icrc1BalanceOfArg {
                owner: super_admin,
                subaccount: None,
            })
            .unwrap(),
        )
        .map(|res| {
            let response = match res {
                WasmResult::Reply(payload) => Decode!(&payload, Nat).unwrap(),
                _ => panic!("\n🛑 icrc_1_balance_of call failed\n"),
            };
            response
        })
        .unwrap();
    ic_cdk::println!("🧪 Result: {:?}", res);

    //
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
    let transfer_args = types::Transaction {
        memo: Some(vec![0]),
        amount: Nat::from(1000000 as u64),
        fee: Some(Nat::from(0 as u64)),
        from_subaccount: None,
        to: types::Recipient {
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
            let response: types::TransferResult = match res {
                WasmResult::Reply(payload) => Decode!(&payload, types::TransferResult).unwrap(),
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
                confirmation_text: Some("GET RICH QUICK".to_string()),
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
                of_principal: Some(PrincipalId(user_principal)),
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
                owner: Some(PrincipalId(user_principal)),
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
            user_principal,
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
            ledger_canister,
            user_principal,
            "icrc1_balance_of",
            candid::encode_one(types::Icrc1BalanceOfArg {
                owner: user_principal,
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

    let expected_balance = Nat::from(60_000_000_000 - tx_fee);
    ic_cdk::println!("🧪 Expected Balance: {:?}", expected_balance);

    deployed_canisters[0].clone()
}
