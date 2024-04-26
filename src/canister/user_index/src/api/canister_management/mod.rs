use candid::Principal;
use ic_cdk::{
    api::{
        call::CallResult,
        is_controller,
        management_canister::{
            main::{canister_status, CanisterInstallMode, CanisterStatusResponse},
            provisional::CanisterIdRecord,
        },
    },
    caller,
};
use ic_cdk_macros::{query, update};
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    common::{
        types::{
            known_principal::KnownPrincipalType,
            wasm::{CanisterWasm, WasmType},
        },
        utils::task::run_task_concurrently,
    },
};

use crate::{
    util::canister_management::{self, reinstall_canister_wasm},
    CANISTER_DATA,
};

pub mod create_pool_of_available_canisters;
pub mod get_subnet_available_capacity;
pub mod get_subnet_backup_capacity;
pub mod recycle_canisters;
pub mod start_upgrades_for_individual_canisters;

#[update]
pub async fn get_user_canister_status(
    canister_id: Principal,
) -> CallResult<(CanisterStatusResponse,)> {
    canister_status(CanisterIdRecord { canister_id }).await
}

#[update]
pub async fn set_permission_to_upgrade_individual_canisters(flag: bool) -> String {
    if !is_controller(&caller()) {
        return "Unauthorized Access".to_string();
    }

    CANISTER_DATA.with(|canister_data_ref| {
        canister_data_ref
            .borrow_mut()
            .allow_upgrades_for_individual_canisters = flag;
    });
    return "Success".to_string();
}

#[query]
pub fn get_list_of_available_canisters() -> Vec<Principal> {
    CANISTER_DATA.with(|canister_data_ref| {
        canister_data_ref
            .borrow()
            .available_canisters
            .clone()
            .into_iter()
            .collect()
    })
}

#[query]
pub fn validate_reset_user_individual_canisters(
    _canisters: Vec<Principal>,
) -> Result<String, String> {
    let caller_id = caller();
    let governance_canister_id = CANISTER_DATA
        .with(|canister_data_ref| {
            canister_data_ref
                .borrow()
                .configuration
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdSnsGovernance)
                .cloned()
        })
        .ok_or("Governance Canister Id not found")?;

    if caller_id != governance_canister_id {
        return Err("This Proposal can only be executed through DAO".to_string());
    };

    Ok("Success".to_string())
}

#[update]
pub async fn reset_user_individual_canisters(canisters: Vec<Principal>) -> Result<String, String> {
    let caller_id = caller();
    let governance_canister_id = CANISTER_DATA
        .with(|canister_data_ref| {
            canister_data_ref
                .borrow()
                .configuration
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdSnsGovernance)
                .cloned()
        })
        .ok_or("Governance Canister Id not found")?;

    if caller_id != governance_canister_id {
        return Err("This method can only be executed through DAO".to_string());
    };

    // Remove profile owner from the data
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.user_principal_id_to_canister_id_map = canister_data
            .user_principal_id_to_canister_id_map
            .iter()
            .filter(|item| !canisters.contains(&item.1))
            .map(|item| (*item.0, *item.1))
            .collect();
    });
    let individual_user_template_canister_wasm = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .wasms
            .get(&WasmType::IndividualUserWasm)
            .unwrap()
            .clone()
    });

    let canister_reinstall_futures = canisters.iter().map(|canister| async {
        match reset_canister(*canister, individual_user_template_canister_wasm.clone()).await {
            Ok(canister_id) => Ok(canister_id),
            Err(e) => Err(e),
        }
    });

    let result_callback =
        |reinstall_res: Result<Principal, (Principal, String)>| match reinstall_res {
            Ok(canister_id) => CANISTER_DATA.with(|canister_data_ref| {
                canister_data_ref
                    .borrow_mut()
                    .available_canisters
                    .insert(canister_id);
            }),
            Err(e) => ic_cdk::println!("Failed to reinstall canister {}", e.1),
        };

    run_task_concurrently(canister_reinstall_futures, 10, result_callback, || false).await;

    Ok(format!("Sucess {}", canisters[0].to_string()))
}

pub async fn reset_canister(
    canister_id: Principal,
    individual_user_template_canister_wasm: CanisterWasm,
) -> Result<Principal, (Principal, String)> {
    canister_management::recharge_canister_if_below_threshold(&canister_id)
        .await
        .map_err(|e| (canister_id, e))?;

    // reinstall wasm
    match reinstall_canister_wasm(
        canister_id,
        None,
        individual_user_template_canister_wasm.version.clone(),
        individual_user_template_canister_wasm.wasm_blob.clone(),
    )
    .await
    {
        Ok(_) => Ok(canister_id),
        Err(e) => Err((canister_id, e)),
    }
}
