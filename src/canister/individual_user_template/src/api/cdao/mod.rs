mod airdrop;
mod token;
use std::collections::{HashMap, HashSet, VecDeque};

use candid::{Encode, Principal};
use futures::{
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt, TryStreamExt,
};
use ic_base_types::PrincipalId;
use ic_cdk::{
    api::{
        call::RejectionCode,
        management_canister::main::{
            create_canister, deposit_cycles, install_code, update_settings, CanisterIdRecord,
            CanisterInstallMode, CanisterSettings, CreateCanisterArgument, InstallCodeArgument,
            UpdateSettingsArgument,
        },
    },
    query, update,
};
use ic_sns_governance::pb::v1::governance::Version as SnsVersion;
use ic_sns_init::{pb::v1::SnsInitPayload, SnsCanisterIds};
use ic_sns_wasm::pb::v1::{GetWasmRequest, GetWasmResponse};
// use ic_sns_swap::pb::v1::{SettleNeuronsFundParticipationRequest, SettleNeuronsFundParticipationResponse};
use ic_nns_governance::neurons_fund::NeuronsFundSnapshot;
use ic_nns_governance::pb::v1::{
    SettleNeuronsFundParticipationRequest, SettleNeuronsFundParticipationResponse,
};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cdao::{AirdropInfo, DeployedCdaoCanisters},
        error::CdaoDeployError,
        session::SessionType,
    },
    common::types::known_principal::KnownPrincipalType,
    constant::{
        MAX_LIMIT_FOR_CREATOR_DAO_SNS_TOKEN, NNS_LEDGER_CANISTER_ID, SNS_TOKEN_ARCHIVE_MODULE_HASH,
        SNS_TOKEN_GOVERNANCE_MODULE_HASH, SNS_TOKEN_INDEX_MODULE_HASH,
        SNS_TOKEN_LEDGER_MODULE_HASH, SNS_TOKEN_ROOT_MODULE_HASH, SNS_TOKEN_SWAP_MODULE_HASH,
        USER_SNS_CANISTER_INITIAL_CYCLES,
    },
};

use crate::{
    util::{
        cycles::request_cycles_from_subnet_orchestrator, subnet_orchestrator::SubnetOrchestrator,
    },
    CANISTER_DATA,
};

pub mod send_creator_dao_stats_to_subnet_orchestrator;
pub mod upgrade_creator_dao_governance_canisters;

#[update]
pub async fn settle_neurons_fund_participation(
    request: SettleNeuronsFundParticipationRequest,
) -> SettleNeuronsFundParticipationResponse {
    let response = Ok(NeuronsFundSnapshot::empty());
    let intermediate = SettleNeuronsFundParticipationResponse::from(response);
    SettleNeuronsFundParticipationResponse::from(intermediate)
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
    CANISTER_DATA.with(|cdata| cdata.borrow().cdao_canisters.clone())
}

#[update]
async fn deploy_cdao_sns(
    init_payload: SnsInitPayload,
    swap_time: u64,
) -> Result<DeployedCdaoCanisters, CdaoDeployError> {
    // * access control
    let current_caller = ic_cdk::caller();
    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id);
    if my_principal_id != Some(current_caller) {
        return Err(CdaoDeployError::Unauthenticated);
    };

    let (registered, limit_hit) = CANISTER_DATA.with(|cdata| {
        let cdata = cdata.borrow();
        let registered = matches!(cdata.session_type, Some(SessionType::RegisteredSession));
        (
            registered,
            cdata.cdao_canisters.len() >= MAX_LIMIT_FOR_CREATOR_DAO_SNS_TOKEN,
        )
    });

    if limit_hit {
        return Err(CdaoDeployError::TokenLimit(
            MAX_LIMIT_FOR_CREATOR_DAO_SNS_TOKEN,
        ));
    }

    // Alloting 0.5T more to the user canister to be on safer side while deploying canisters
    request_cycles_from_subnet_orchestrator(6 * USER_SNS_CANISTER_INITIAL_CYCLES)
        .await
        .map_err(|e| CdaoDeployError::CycleError(e))?;

    let subnet_orchestrator = SubnetOrchestrator::new()
        .map_err(|e| CdaoDeployError::CallError(RejectionCode::CanisterError, e))?;

    let canister_ids: Vec<Principal> = (0..5)
        .map(|_| subnet_orchestrator.get_empty_canister())
        .collect::<FuturesOrdered<_>>()
        .try_collect()
        .await
        .map_err(|e| CdaoDeployError::CallError(RejectionCode::CanisterError, e))?;

    canister_ids
        .iter()
        .map(|canister_id| {
            deposit_cycles(
                CanisterIdRecord {
                    canister_id: *canister_id,
                },
                USER_SNS_CANISTER_INITIAL_CYCLES,
            )
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect()
        .await?;

    let canisters: Vec<PrincipalId> = canister_ids
        .into_iter()
        .map(|canister_id| PrincipalId::from(canister_id))
        .collect();

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

    let gov_hash = hex::decode(SNS_TOKEN_GOVERNANCE_MODULE_HASH).unwrap();
    let ledger_hash = hex::decode(SNS_TOKEN_LEDGER_MODULE_HASH).unwrap();
    let root_hash = hex::decode(SNS_TOKEN_ROOT_MODULE_HASH).unwrap();
    let swap_hash = hex::decode(SNS_TOKEN_SWAP_MODULE_HASH).unwrap();
    let index_hash = hex::decode(SNS_TOKEN_INDEX_MODULE_HASH).unwrap();
    let arhive_hash = hex::decode(SNS_TOKEN_ARCHIVE_MODULE_HASH).unwrap();

    let sns_version = SnsVersion {
        governance_wasm_hash: gov_hash.clone(),
        ledger_wasm_hash: ledger_hash.clone(),
        root_wasm_hash: root_hash.clone(),
        swap_wasm_hash: swap_hash.clone(),
        index_wasm_hash: index_hash.clone(),
        archive_wasm_hash: arhive_hash.clone(),
    };

    let mut payloads = init_payload
        .build_canister_payloads(&sns_canisters, Some(sns_version), true)
        .map_err(CdaoDeployError::InvalidInitPayload)?;
    let time_seconds = ic_cdk::api::time() / 1_000_000_000;
    payloads.swap.swap_start_timestamp_seconds = Some(time_seconds);
    payloads.swap.swap_due_timestamp_seconds = Some(time_seconds + swap_time);
    payloads.swap.icp_ledger_canister_id = NNS_LEDGER_CANISTER_ID.into();
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

    ic_cdk::println!("gov_hash: {:?}", gov_hash);

    let mut wasm_bins: VecDeque<_> = [gov_hash, ledger_hash, root_hash, swap_hash, index_hash]
        .into_iter()
        .map(|hash| async move {
            let req = GetWasmRequest { hash };
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

    sns_install_futs.push(install_canister_wasm(
        wasm_bins.pop_front().unwrap(),
        Encode!(&payloads.index_ng).unwrap(),
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
        airdrop_info: AirdropInfo {
            principals_who_successfully_claimed: HashMap::new(),
        },
    };
    CANISTER_DATA.with(|cdata| {
        let mut cdata = cdata.borrow_mut();
        cdata.cdao_canisters.push(deployed_cans.clone());
        cdata.token_roots.insert(root.0, ());
    });

    let send_creator_dao_stats_res =
        subnet_orchestrator.send_creator_dao_stats(vec![root.into()].into_iter().collect());

    if let Err(e) = send_creator_dao_stats_res {
        ic_cdk::println!("Error sending creator stats to subnet orchestrator {}", e)
    }

    Ok(deployed_cans)
}
