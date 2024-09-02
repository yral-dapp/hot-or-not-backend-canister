mod token;

use std::collections::{HashMap, VecDeque};

use candid::{Encode, Principal};
use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    TryStreamExt,
};
use ic_base_types::PrincipalId;
use ic_cdk::{
    api::management_canister::main::{
        create_canister, install_code, update_settings, CanisterInstallMode, CanisterSettings,
        CreateCanisterArgument, InstallCodeArgument, UpdateSettingsArgument,
    }, query, update
};
use ic_sns_init::{pb::v1::SnsInitPayload, SnsCanisterIds};
use ic_sns_wasm::pb::v1::{GetWasmResponse, GetWasmRequest};
// use ic_sns_swap::pb::v1::{SettleNeuronsFundParticipationRequest, SettleNeuronsFundParticipationResponse};
use ic_nns_governance::neurons_fund::NeuronsFundSnapshot;
use ic_nns_governance::pb::v1::{SettleNeuronsFundParticipationRequest, SettleNeuronsFundParticipationResponse};
use shared_utils::{
    canister_specific::individual_user_template::{consts::CDAO_TOKEN_LIMIT, types::{
        cdao::DeployedCdaoCanisters, error::CdaoDeployError, session::SessionType,
    }},
    common::types::known_principal::KnownPrincipalType,
};

use crate::CANISTER_DATA;
// 0.1T
const CDAO_CYCLE_CNT: u128 = 100000000000;

#[update]
pub async fn settle_neurons_fund_participation(
    request: SettleNeuronsFundParticipationRequest,
) -> SettleNeuronsFundParticipationResponse {
    let response = Ok(NeuronsFundSnapshot::empty());
    let intermediate = SettleNeuronsFundParticipationResponse::from(response);
    SettleNeuronsFundParticipationResponse::from(intermediate)
}

async fn create_empty_canister(
    arg: CreateCanisterArgument,
) -> Result<PrincipalId, CdaoDeployError> {
    let can = create_canister(arg, CDAO_CYCLE_CNT).await?;
    Ok(PrincipalId(can.0.canister_id))
}

async fn install_canister_wasm(
    wasm: Vec<u8>,
    arg: Vec<u8>,
    canister_id: PrincipalId,
) -> Result<(), CdaoDeployError> {
    let install_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: canister_id.0,
        wasm_module: wasm,
        arg,
    };
    install_code(install_arg).await?;
    Ok(())
}

async fn update_controllers(
    canister_id: PrincipalId,
    controllers: Vec<Principal>,
) -> Result<(), CdaoDeployError> {
    update_settings(UpdateSettingsArgument {
        canister_id: canister_id.0,
        settings: CanisterSettings {
            controllers: Some(controllers),
            ..Default::default()
        },
    })
    .await?;
    Ok(())
}

#[query]
async fn deployed_cdao_canisters() -> Vec<DeployedCdaoCanisters> {
    CANISTER_DATA.with(|cdata| {
        cdata
            .borrow()
            .cdao_canisters
            .clone()
    })
}

#[update]
async fn deploy_cdao_sns(
    init_payload: SnsInitPayload,
) -> Result<DeployedCdaoCanisters, CdaoDeployError> { 
    let (registered, limit_hit) = CANISTER_DATA.with(|cdata| {
        let cdata = cdata.borrow();
        let registered = matches!(cdata.session_type, Some(SessionType::RegisteredSession));
        (registered, cdata.cdao_canisters.len() == CDAO_TOKEN_LIMIT)
    });
    /*if !registered {
        return Err(CdaoDeployError::Unregistered);
    }*/
    if limit_hit {
        return Err(CdaoDeployError::TokenLimit(CDAO_TOKEN_LIMIT));
    }

    let creation_arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![ic_cdk::id()]),
            ..Default::default()
        }),
    };

    let canisters: Vec<_> = (0..5)
        .map(|_| create_empty_canister(creation_arg.clone()))
        .collect::<FuturesOrdered<_>>()
        .try_collect()
        .await?;
    let governance = canisters[0];
    let ledger = canisters[1];
    let root = canisters[2];
    let swap = canisters[3];
    let index = canisters[4];

    let sns_canisters = SnsCanisterIds {
        governance,
        ledger,
        root,
        swap,
        index,
    };
    let mut payloads = init_payload
        .build_canister_payloads(&sns_canisters, None, true)
        .map_err(CdaoDeployError::InvalidInitPayload)?;
    let time_seconds = ic_cdk::api::time() / 1_000_000_000;
    payloads.swap.swap_start_timestamp_seconds = Some(time_seconds);
    payloads.swap.swap_due_timestamp_seconds = Some(time_seconds + 300);
    payloads.swap.icp_ledger_canister_id = "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string();
    payloads.swap.nns_governance_canister_id = ic_cdk::id().to_string();
    
    let sns_wasm = CANISTER_DATA
        .with(|cdata| {
            cdata
                .borrow()
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdSnsWasm)
                .copied()
        })
        .expect("SNS WASM not specified in config");

    let sns_versions = ic_cdk::call::<_, (HashMap<String, String>,)>(
        sns_wasm,
        "get_latest_sns_version_pretty",
        (),
    )
    .await?
    .0;
    let get_wasm_hash = |name: &str| hex::decode(&sns_versions[name]).unwrap();
    let gov_hash = get_wasm_hash("Governance");
    let ledger_hash = get_wasm_hash("Ledger");
    let root_hash = get_wasm_hash("Root");
    let swap_hash = get_wasm_hash("Swap");
    let index_hash = get_wasm_hash("Ledger Index");

    let mut wasm_bins: VecDeque<_> = [gov_hash, ledger_hash, root_hash, swap_hash, index_hash]
        .into_iter()
        .map(|hash| async move {
            let req = GetWasmRequest {
                hash,
            };
            let wasm_res =
                ic_cdk::call::<_, (GetWasmResponse,)>(sns_wasm, "get_wasm", (req,)).await?;
            Ok::<_, CdaoDeployError>(wasm_res.0.wasm.unwrap().wasm)
        })
        .collect::<FuturesOrdered<_>>()
        .try_collect()
        .await?;

    let mut sns_install_futs = FuturesUnordered::new();
    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.governance).unwrap(),
        governance,
    ));
    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.ledger).unwrap(),
        ledger,
    ));
    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.root).unwrap(),
        root,
    ));
    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.swap).unwrap(),
        swap,
    ));

    let index_init = match payloads.index_ng {
        Some(ic_icrc1_index_ng::IndexArg::Init(init)) => Encode!(&ic_icrc1_index::InitArgs {
            ledger_id: PrincipalId(init.ledger_id).try_into().unwrap(),
        })
        .unwrap(),
        _ => panic!("Index init not specified?!"),
    };
    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        index_init,
        index,
    ));
    while sns_install_futs.try_next().await?.is_some() {}

    let admin_canister = CANISTER_DATA
        .with(|cdata| {
            cdata
                .borrow()
                .known_principal_ids
                .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
                .copied()
        })
        .expect("Super admin not specified in config");

    let user_can = ic_cdk::id();
    let mut update_ctrls_futs = FuturesUnordered::new();
    update_ctrls_futs.push(update_controllers(
        governance,
        vec![admin_canister, user_can, root.0],
    ));
    update_ctrls_futs.push(update_controllers(
        root,
        vec![admin_canister, user_can, governance.0],
    ));
    update_ctrls_futs.push(update_controllers(
        ledger,
        vec![admin_canister, user_can, governance.0],
    ));
    update_ctrls_futs.push(update_controllers(
        swap,
        vec![
            admin_canister,
            user_can,
            swap.0,
            ic_nns_constants::ROOT_CANISTER_ID.into(),
        ],
    ));
    update_ctrls_futs.push(update_controllers(
        index,
        vec![admin_canister, user_can, root.0],
    ));
    while update_ctrls_futs.try_next().await?.is_some() {}

    let deployed_cans = DeployedCdaoCanisters {
        governance: governance.0,
        ledger: ledger.0,
        root: root.0,
        swap: swap.0,
        index: index.0,
    };
    CANISTER_DATA.with(|cdata| {
        let mut cdata = cdata.borrow_mut();
        cdata.cdao_canisters.push(deployed_cans);
        cdata.token_roots.insert(root.0, ());
    });

    Ok(deployed_cans)
}
