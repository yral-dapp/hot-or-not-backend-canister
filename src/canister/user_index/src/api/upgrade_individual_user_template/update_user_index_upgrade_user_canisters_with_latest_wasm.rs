use ic_cdk::api::management_canister::{
    main::{self, CanisterInstallMode},
    provisional::CanisterIdRecord,
};
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    common::{types::known_principal::KnownPrincipalType, utils::system_time},
    constant::{DYNAMIC_CANISTER_DEFAULT_CREATION_BALANCE, MINIMUM_CYCLES_TO_REVIVE_CANISTER},
};

use crate::{
    data_model::canister_upgrade::upgrade_status::UpgradeStatus, util::canister_management,
    CANISTER_DATA,
};

#[ic_cdk::update]
#[candid::candid_method(update)]
async fn update_user_index_upgrade_user_canisters_with_latest_wasm() -> String {
    let api_caller = ic_cdk::caller();

    let known_principal_ids = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().known_principal_ids.clone());

    if known_principal_ids
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .unwrap()
        .clone()
        != api_caller
    {
        return "Unauthorized caller".to_string();
    };

    let mut upgrade_count = 0;
    let mut failed_canister_ids = Vec::new();

    let user_principal_id_to_canister_id_map = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_canister_id_map
            .clone()
    });

    let saved_upgrade_status = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .last_run_upgrade_status
            .clone()
    });

    for (user_principal_id, user_canister_id) in user_principal_id_to_canister_id_map.iter() {
        match canister_management::upgrade_individual_user_canister(
            user_canister_id.clone(),
            CanisterInstallMode::Upgrade,
            IndividualUserTemplateInitArgs {
                known_principal_ids: None,
                profile_owner: None,
                upgrade_version_number: Some(saved_upgrade_status.version_number + 1),
            },
        )
        .await
        {
            Ok(_) => {
                upgrade_count += 1;
            }
            Err(e) => {
                let response_result = main::canister_status(CanisterIdRecord {
                    canister_id: user_canister_id.clone(),
                })
                .await;

                if response_result.is_err() {
                    main::deposit_cycles(
                        CanisterIdRecord {
                            canister_id: user_canister_id.clone(),
                        },
                        DYNAMIC_CANISTER_DEFAULT_CREATION_BALANCE,
                    )
                    .await
                    .unwrap();
                }

                match canister_management::upgrade_individual_user_canister(
                    user_canister_id.clone(),
                    CanisterInstallMode::Upgrade,
                    IndividualUserTemplateInitArgs {
                        known_principal_ids: None,
                        profile_owner: None,
                        upgrade_version_number: Some(saved_upgrade_status.version_number + 1),
                    },
                )
                .await
                {
                    Ok(_) => {
                        upgrade_count += 1;
                    }
                    Err(_) => {
                        ic_cdk::print(format!(
                            "🥫 Failed to upgrade canister {:?} belonging to user {:?} with error: {:?}",
                            user_canister_id.to_text(),
                            user_principal_id.to_text(),
                            e
                        ));
                        failed_canister_ids
                            .push((user_principal_id.clone(), user_canister_id.clone()));
                    }
                };
            }
        }

        // * Enable for data backup
        // let upgrade_response: CallResult<()> = call::call(
        //     user_canister_id.clone(),
        //     "backup_data_to_backup_canister",
        //     (user_principal_id.clone(), user_canister_id.clone()),
        // )
        // .await;
        // upgrade_response.ok();

        CANISTER_DATA.with(|canister_data_ref_cell| {
            let mut last_run_upgrade_status = canister_data_ref_cell
                .borrow_mut()
                .last_run_upgrade_status
                .clone();

            last_run_upgrade_status.successful_upgrade_count = upgrade_count;
            last_run_upgrade_status.failed_canister_ids = failed_canister_ids.clone();

            canister_data_ref_cell.borrow_mut().last_run_upgrade_status = last_run_upgrade_status;
        });
    }

    let new_upgrade_status = UpgradeStatus {
        version_number: saved_upgrade_status.version_number + 1,
        last_run_on: system_time::get_current_system_time_from_ic(),
        successful_upgrade_count: upgrade_count,
        failed_canister_ids,
    };

    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell.borrow_mut().last_run_upgrade_status = new_upgrade_status.clone();
    });

    return new_upgrade_status.to_string();
}
