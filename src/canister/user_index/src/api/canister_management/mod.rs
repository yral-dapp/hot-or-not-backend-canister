use ic_cdk::{api::{call::CallResult, is_controller, management_canister::{main::{canister_status, CanisterStatusResponse, CanisterInstallMode}, provisional::CanisterIdRecord}}, caller};
use candid::Principal;
use ic_cdk_macros::{query, update};
use shared_utils::{canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs, common::{types::{known_principal::KnownPrincipalType, wasm::{CanisterWasm, WasmType}}, utils::task::run_task_concurrently}};

use crate::{CANISTER_DATA, util::canister_management};

use super::upgrade_individual_user_template::update_user_index_upgrade_user_canisters_with_latest_wasm;

pub mod create_pool_of_available_canisters;
pub mod get_subnet_available_capacity;
pub mod get_subnet_backup_capacity;


#[update]
pub async fn get_user_canister_status(canister_id: Principal) -> CallResult<(CanisterStatusResponse,)>{
    canister_status(CanisterIdRecord {canister_id}).await
}


#[update]
pub async fn set_permission_to_upgrade_individual_canisters(flag: bool) -> String {
    
    if !is_controller(&caller()) {
        return "Unauthorized Access".to_string();
    }

    CANISTER_DATA.with(|canister_data_ref| {
       canister_data_ref.borrow_mut().allow_upgrades_for_individual_canisters = flag;
    });
    return "Success".to_string()
}

#[update]
async fn start_upgrades_for_individual_canisters(version: String, individual_user_wasm: Vec<u8>) -> String {
    
    if !is_controller(&caller()) {
        panic!("Unauthorized caller");
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.allow_upgrades_for_individual_canisters = true;
        canister_data.last_run_upgrade_status.version = version.clone();
        let canister_wasm = CanisterWasm {
            version: version.clone(),
            wasm_blob: individual_user_wasm.clone()
        };
        canister_data.wasms.insert(WasmType::IndividualUserWasm, canister_wasm);
    });
    ic_cdk::spawn(update_user_index_upgrade_user_canisters_with_latest_wasm::upgrade_user_canisters_with_latest_wasm(version, individual_user_wasm));
    "Success".to_string()
}

#[query]
pub fn get_list_of_available_canisters() -> Vec<Principal> {
    CANISTER_DATA.with(|canister_data_ref|{
        canister_data_ref.borrow().available_canisters.clone().into_iter().collect()
    })
}

#[query]
pub fn validate_reset_user_individual_canisters(_canisters: Vec<Principal>) -> Result<String, String> {
    let caller_id = caller();
    let governance_canister_id = CANISTER_DATA.with(|canister_data_ref| {
        canister_data_ref.borrow().configuration.known_principal_ids.get(&KnownPrincipalType::CanisterIdSnsGovernance).cloned()
    }).ok_or("Governance Canister Id not found")?;
    
    if caller_id !=  governance_canister_id {
        return Err("This Proposal can only be executed through DAO".to_string())
    };

    Ok("Success".to_string())
}


#[update]
pub async fn reset_user_individual_canisters(canisters: Vec<Principal>) -> Result<String, String> {
    let caller_id = caller();
    let governance_canister_id = CANISTER_DATA.with(|canister_data_ref| {
        canister_data_ref.borrow().configuration.known_principal_ids.get(&KnownPrincipalType::CanisterIdSnsGovernance).cloned()
    }).ok_or("Governance Canister Id not found")?;
    
    if caller_id !=  governance_canister_id {
        return Err("This method can only be executed through DAO".to_string())
    };

    // Remove profile owner from the data
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.user_principal_id_to_canister_id_map = canister_data.user_principal_id_to_canister_id_map
            .iter()
            .filter(|item| !canisters.contains(&item.1))
            .map(|item| (*item.0, *item.1))
            .collect();
    });
    
    let canister_reinstall_futures = canisters.iter().map(|canister| async move {
        canister_management::recharge_canister_if_below_threshold(&canister).await.map_err(|e| (*canister, e))?;
        canister_management::upgrade_individual_user_canister(canister.clone(), CanisterInstallMode::Reinstall, IndividualUserTemplateInitArgs {
            known_principal_ids: Some(CANISTER_DATA.with(|canister_data_ref| {canister_data_ref.borrow().configuration.known_principal_ids.clone()})),
            profile_owner: None,
            upgrade_version_number: Some(CANISTER_DATA.with(|canister_data_ref| canister_data_ref.borrow().last_run_upgrade_status.version_number)),
            url_to_send_canister_metrics_to: Some(CANISTER_DATA.with(|canister_data_ref| canister_data_ref.borrow().configuration.url_to_send_canister_metrics_to.clone())),
            version: CANISTER_DATA.with(|canister_data_ref_cell| canister_data_ref_cell.borrow().last_run_upgrade_status.version.clone())
        },
        CANISTER_DATA.with_borrow(|canister_data| canister_data.wasms.get(&WasmType::IndividualUserWasm).unwrap().wasm_blob.clone()))
        .await
        .map_err(|e| (*canister, e.1))?;
        Ok(*canister)
    });

    let result_callback = |reinstall_res: Result<Principal, (Principal, String)>| { 
        
       match reinstall_res {
        Ok(canister_id) => {
            CANISTER_DATA.with(|canister_data_ref| {
                canister_data_ref.borrow_mut().available_canisters.insert(canister_id);
            })
        },
        Err(e) => ic_cdk::println!("Failed to reinstall canister {}", e.1)
       }
    };


    run_task_concurrently(canister_reinstall_futures, 10, result_callback, || false).await;

    Ok(format!("Sucess {}", canisters[0].to_string()))
}