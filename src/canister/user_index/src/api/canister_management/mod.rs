use ic_cdk::api::{management_canister::{main::{canister_status, CanisterStatusResponse}, provisional::CanisterIdRecord}, call::CallResult};
use candid::Principal;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

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