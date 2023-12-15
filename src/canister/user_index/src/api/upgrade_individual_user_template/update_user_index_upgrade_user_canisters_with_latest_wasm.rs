use std::time::SystemTime;

use candid::Principal;
use ic_cdk::api::management_canister::{
    main::{self, CanisterInstallMode},
    provisional::CanisterIdRecord,
};
use shared_utils::{
    canister_specific::individual_user_template::types::arg::IndividualUserTemplateInitArgs,
    common::utils::system_time,
    constant::{CYCLES_THRESHOLD_TO_INITIATE_RECHARGE, INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT},
};

use crate::{
    data_model::{configuration::Configuration, CanisterData},
    util::canister_management,
    CANISTER_DATA,
};

pub async fn upgrade_user_canisters_with_latest_wasm() {
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

    let configuration = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().configuration.clone());

    for (user_principal_id, user_canister_id) in user_principal_id_to_canister_id_map.iter() {
        let is_canister_below_threshold_balance =
            is_canister_below_threshold_balance(user_canister_id).await;

        if is_canister_below_threshold_balance {
            let recharge_result = recharge_canister(user_canister_id).await;

            if recharge_result.is_err() {
                let err = recharge_result.err().unwrap();
                failed_canister_ids.push((*user_principal_id, *user_canister_id, err));
                continue;
            }
        }

        let upgrade_result = upgrade_user_canister(
            user_principal_id,
            user_canister_id,
            saved_upgrade_status.version_number,
            &configuration,
        )
        .await;

        if upgrade_result.is_err() {
            let err = upgrade_result.err().unwrap();
            ic_cdk::print(format!(
                "Failed to upgrade canister: {:?} with error: {:?}",
                user_canister_id.to_text(),
                err
            ));
            failed_canister_ids.push((*user_principal_id, *user_canister_id, err));
            continue;
        }

        upgrade_count += 1;

        // * Enable for data backup
        // let upgrade_response: CallResult<()> = call::call(
        //     user_canister_id.clone(),
        //     "backup_data_to_backup_canister",
        //     (user_principal_id.clone(), user_canister_id.clone()),
        // )
        // .await;
        // upgrade_response.ok();

        CANISTER_DATA.with(|canister_data_ref_cell| {
            update_upgrade_status(
                &mut canister_data_ref_cell.borrow_mut(),
                upgrade_count,
                &failed_canister_ids,
                None,
                None,
            );
        });
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        update_upgrade_status(
            &mut canister_data_ref_cell.borrow_mut(),
            upgrade_count,
            &failed_canister_ids,
            Some(saved_upgrade_status.version_number + 1),
            Some(system_time::get_current_system_time_from_ic()),
        );
    });
}

async fn is_canister_below_threshold_balance(canister_id: &Principal) -> bool {
    let response: Result<(u128,), (_, _)> =
        ic_cdk::call(*canister_id, "get_user_caniser_cycle_balance", ()).await;

    if response.is_err() {
        return true;
    }

    let (balance,): (u128,) = response.unwrap();

    if balance < CYCLES_THRESHOLD_TO_INITIATE_RECHARGE {
        return true;
    }

    false
}

async fn recharge_canister(canister_id: &Principal) -> Result<(), String> {
    main::deposit_cycles(
        CanisterIdRecord {
            canister_id: *canister_id,
        },
        INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT,
    )
    .await
    .map_err(|e| e.1)
}

async fn upgrade_user_canister(
    user_principal_id: &Principal,
    canister_id: &Principal,
    version_number: u64,
    configuration: &Configuration,
) -> Result<(), String> {
    canister_management::upgrade_individual_user_canister(
        *canister_id,
        CanisterInstallMode::Upgrade,
        IndividualUserTemplateInitArgs {
            known_principal_ids: Some(configuration.known_principal_ids.clone()),
            profile_owner: Some(*user_principal_id),
            upgrade_version_number: Some(version_number + 1),
            url_to_send_canister_metrics_to: Some(
                configuration.url_to_send_canister_metrics_to.clone(),
            ),
        },
        true
    )
    .await
    .map_err(|e| e.1)
}

fn update_upgrade_status(
    canister_data: &mut CanisterData,
    upgrade_count: u32,
    failed_canister_ids: &[(Principal, Principal, String)],
    version_number: Option<u64>,
    last_run_on: Option<SystemTime>,
) {
    let mut last_run_upgrade_status = canister_data.last_run_upgrade_status.clone();

    last_run_upgrade_status.successful_upgrade_count = upgrade_count;
    last_run_upgrade_status.failed_canister_ids = failed_canister_ids.to_owned();
    last_run_upgrade_status.version_number =
        version_number.unwrap_or(canister_data.last_run_upgrade_status.version_number);
    last_run_upgrade_status.last_run_on =
        last_run_on.unwrap_or(canister_data.last_run_upgrade_status.last_run_on);

    canister_data.last_run_upgrade_status = last_run_upgrade_status;
}
