use ic_cdk::api::{
    call::{self, CallResult},
    management_canister::{
        main::{self, CanisterInstallMode},
        provisional::CanisterIdRecord,
    },
};
use shared_utils::{
    access_control::{self, UserAccessRole},
    constant::MINIMUM_CYCLES_TO_REVIVE_CANISTER,
    date_time::system_time,
};

use crate::{
    data_model::canister_upgrade::upgrade_status::UpgradeStatusV1, util::canister_management,
    CANISTER_DATA,
};

#[ic_cdk::update]
#[candid::candid_method(update)]
async fn update_user_index_upgrade_user_canisters_with_latest_wasm() {
    let api_caller = ic_cdk::caller();

    let access_control_map = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().access_control_map.clone());

    // TODO: update the return type of this method so that unauthorized callers are informed accordingly
    if !access_control::does_principal_have_role_v2(
        &access_control_map,
        UserAccessRole::CanisterAdmin,
        api_caller,
    ) {
        panic!("Unauthorized caller");
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
            saved_upgrade_status.version_number + 1,
        )
        .await
        {
            Ok(_) => {
                upgrade_count += 1;
            }
            Err(e) => {
                ic_cdk::print(format!(
                    "🥫 Failed to upgrade canister {:?} belonging to user {:?} with error: {:?}",
                    user_canister_id.to_text(),
                    user_principal_id.to_text(),
                    e
                ));

                // TODO: update schema to accept failure reason
                failed_canister_ids.push((user_principal_id.clone(), user_canister_id.clone()));

                let response_result = main::canister_status(CanisterIdRecord {
                    canister_id: user_canister_id.clone(),
                })
                .await;

                if response_result.is_err() {
                    main::deposit_cycles(
                        CanisterIdRecord {
                            canister_id: user_canister_id.clone(),
                        },
                        MINIMUM_CYCLES_TO_REVIVE_CANISTER,
                    )
                    .await
                    .unwrap();
                }

                canister_management::upgrade_individual_user_canister(
                    user_canister_id.clone(),
                    CanisterInstallMode::Upgrade,
                    saved_upgrade_status.version_number + 1,
                )
                .await
                .ok();
            }
        }

        let upgrade_response: CallResult<()> = call::call(
            user_canister_id.clone(),
            "backup_data_to_backup_canister",
            (user_principal_id.clone(), user_canister_id.clone()),
        )
        .await;
        upgrade_response.ok();

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

    let new_upgrade_status = UpgradeStatusV1 {
        version_number: saved_upgrade_status.version_number + 1,
        last_run_on: system_time::get_current_system_time_from_ic(),
        successful_upgrade_count: upgrade_count,
        failed_canister_ids,
    };

    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell.borrow_mut().last_run_upgrade_status = new_upgrade_status;
    });
}
