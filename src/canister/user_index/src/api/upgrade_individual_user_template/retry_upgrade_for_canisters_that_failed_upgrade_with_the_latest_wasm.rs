use candid::Principal;
use ic_cdk::api::management_canister::main::CanisterInstallMode;
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    common::types::known_principal::KnownPrincipalType,
};

use crate::{util::canister_management, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
pub async fn retry_upgrade_for_canisters_that_failed_upgrade_with_the_latest_wasm(
) -> Vec<(Principal, Principal, String)> {
    let api_caller = ic_cdk::caller();

    let known_principal_ids = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().known_principal_ids.clone());

    if known_principal_ids
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .unwrap()
        .clone()
        != api_caller
    {
        return vec![(
            Principal::anonymous(),
            Principal::anonymous(),
            "Unauthorized caller".to_string(),
        )];
    };

    let mut status_to_return = Vec::new();

    let saved_upgrade_status = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .last_run_upgrade_status
            .clone()
    });

    for (user_principal_id, user_canister_id) in saved_upgrade_status.failed_canister_ids.iter() {
        match canister_management::upgrade_individual_user_canister(
            *user_canister_id,
            CanisterInstallMode::Upgrade,
            IndividualUserTemplateInitArgs {
                known_principal_ids: Some(CANISTER_DATA.with(|canister_data_ref_cell| {
                    canister_data_ref_cell.borrow().known_principal_ids.clone()
                })),
                profile_owner: Some(*user_principal_id),
                upgrade_version_number: Some(saved_upgrade_status.version_number + 1),
            },
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                status_to_return.push((*user_principal_id, *user_canister_id, e.1));
            }
        }
    }

    return status_to_return;
}
