
use ic_cdk::{api::{management_canister::{main::{canister_status, CanisterStatusResponse, CanisterInstallMode}, provisional::CanisterIdRecord}, call::CallResult}, caller};
use candid::Principal;
use shared_utils::{common::{types::known_principal::KnownPrincipalType, utils::task::run_task_concurrently}, canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs};

use crate::{CANISTER_DATA, util::canister_management};

use super::upgrade_individual_user_template::update_user_index_upgrade_user_canisters_with_latest_wasm;



#[candid::candid_method(update)]
#[ic_cdk::update]
pub async fn get_user_canister_status(canister_id: Principal) -> CallResult<(CanisterStatusResponse,)>{
    canister_status(CanisterIdRecord {canister_id}).await
}


#[candid::candid_method(update)]
#[ic_cdk::update]
pub async fn set_permission_to_upgrade_individual_canisters(flag: bool) -> String {
    let api_caller = ic_cdk::caller();
    let known_principal_ids = CANISTER_DATA.with(|canister_data_ref_cell| canister_data_ref_cell.borrow().known_principal_ids.clone());
    if *known_principal_ids
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .unwrap()
        != api_caller
    {
        return "Unauthorized caller".to_string();
    };

    CANISTER_DATA.with(|canister_data_ref| {
       canister_data_ref.borrow_mut().allow_upgrades_for_individual_canisters = flag;
    });
    return "Success".to_string()
}

#[candid::candid_method(update)]
#[ic_cdk::update]
pub async fn start_upgrades_for_individual_canisters() -> String {
    let api_caller = ic_cdk::caller();
    let known_principal_ids = CANISTER_DATA.with(|canister_data_ref_cell| canister_data_ref_cell.borrow().known_principal_ids.clone());
    if *known_principal_ids
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .unwrap()
        != api_caller
    {
        return "Unauthorized caller".to_string();
    };

    CANISTER_DATA.with(|canister_data_ref| {
        canister_data_ref.borrow_mut().allow_upgrades_for_individual_canisters = true;
    });
    ic_cdk::spawn(update_user_index_upgrade_user_canisters_with_latest_wasm::upgrade_user_canisters_with_latest_wasm());
    "Success".to_string()
}

#[ic_cdk::query]
#[candid::candid_method(query)]
pub fn get_list_of_available_canisters() -> Vec<Principal> {
    CANISTER_DATA.with(|canister_data_ref|{
        canister_data_ref.borrow().available_canisters.clone().into_iter().collect()
    })
}

#[ic_cdk::query]
#[candid::candid_method(query)]
pub fn validate_reset_user_individual_canisters(_canisters: Vec<Principal>) -> Result<String, String> {
    let caller_id = caller();
    let governance_canister_id = CANISTER_DATA.with(|canister_data_ref| {
        canister_data_ref.borrow().known_principal_ids.get(&KnownPrincipalType::CanisterIdSnsGovernance).cloned()
    }).ok_or("Governance Canister Id not found")?;
    
    if caller_id !=  governance_canister_id {
        return Err("This Proposal can only be executed through DAO".to_string())
    };

    Ok("Success".to_string())
}


#[ic_cdk::update]
#[candid::candid_method(update)]
pub async fn reset_user_individual_canisters(canisters: Vec<Principal>) -> Result<String, String> {
    let caller_id = caller();
    let governance_canister_id = CANISTER_DATA.with(|canister_data_ref| {
        canister_data_ref.borrow().known_principal_ids.get(&KnownPrincipalType::CanisterIdSnsGovernance).cloned()
    }).ok_or("Governance Canister Id not found")?;
    
    if caller_id !=  governance_canister_id {
        return Err("This method can only be executed through DAO".to_string())
    };
    
    let canister_reinstall_futures = canisters.iter().map(|canister| async {
        canister_management::upgrade_individual_user_canister(canister.clone(), CanisterInstallMode::Reinstall, IndividualUserTemplateInitArgs {
            known_principal_ids: Some(CANISTER_DATA.with(|canister_data_ref| {canister_data_ref.borrow().known_principal_ids.clone()})),
            profile_owner: None,
            upgrade_version_number: Some(CANISTER_DATA.with(|canister_data_ref| canister_data_ref.borrow().last_run_upgrade_status.version_number)),
            url_to_send_canister_metrics_to: Some(CANISTER_DATA.with(|canister_data_ref| canister_data_ref.borrow().configuration.url_to_send_canister_metrics_to.clone())),
            version: CANISTER_DATA.with(|canister_data_ref_cell| canister_data_ref_cell.borrow().last_run_upgrade_status.version.clone())
        }, false).await?;
        Ok(canister.clone())
    });

    let result_callback = |reinstall_res: CallResult<Principal>| { 
        
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