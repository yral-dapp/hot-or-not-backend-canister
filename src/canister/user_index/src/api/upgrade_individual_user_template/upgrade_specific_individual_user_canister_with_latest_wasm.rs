use candid::Principal;
use ic_cdk::api::management_canister::main::CanisterInstallMode;
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    common::types::known_principal::KnownPrincipalType,
};

use crate::{util::canister_management, CANISTER_DATA};

// * dfx canister call user_index upgrade_specific_individual_user_canister_with_latest_wasm '(principal "", principal "", null)' --network ic

#[ic_cdk::update]
#[candid::candid_method(update)]
async fn upgrade_specific_individual_user_canister_with_latest_wasm(
    user_principal_id: Principal,
    user_canister_id: Principal,
    upgrade_mode: Option<CanisterInstallMode>,
    unsafe_drop_stable_memory: bool
) -> String {
    let api_caller = ic_cdk::caller();

    let known_principal_ids = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().configuration.known_principal_ids.clone());

    if *known_principal_ids
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .unwrap()
        != api_caller
    {
        return "Unauthorized caller".to_string();
    };

    let saved_upgrade_status = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .last_run_upgrade_status
            .clone()
    });

    let configuration = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().configuration.clone());

    match canister_management::upgrade_individual_user_canister(
        user_canister_id,
        upgrade_mode.unwrap_or(CanisterInstallMode::Upgrade),
        IndividualUserTemplateInitArgs {
            known_principal_ids: Some(known_principal_ids.clone()),
            profile_owner: Some(user_principal_id),
            upgrade_version_number: Some(saved_upgrade_status.version_number + 1),
            url_to_send_canister_metrics_to: Some(configuration.url_to_send_canister_metrics_to),
            version: saved_upgrade_status.version
        },
        unsafe_drop_stable_memory
    )
    .await
    {
        Ok(_) => "Success".to_string(),
        Err(e) => e.1,
    }
}
