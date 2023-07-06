use std::time::Duration;

use ic_cdk::api::management_canister::{
    main::UpdateSettingsArgument, provisional::CanisterSettings,
};
use shared_utils::common::utils::stable_memory_serializer_deserializer;

use crate::{
    api::{
        upgrade_individual_user_template::update_user_index_upgrade_user_canisters_with_latest_wasm,
        well_known_principal::update_locally_stored_well_known_principals,
    },
    data_model::CanisterData,
    CANISTER_DATA,
};

use super::pre_upgrade::BUFFER_SIZE_BYTES;

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    refetch_well_known_principals();
    // upgrade_all_indexed_user_canisters();
    remove_global_admin_as_controller();
}

fn restore_data_from_stable_memory() {
    match stable_memory_serializer_deserializer::deserialize_from_stable_memory::<CanisterData>(
        BUFFER_SIZE_BYTES,
    ) {
        Ok(canister_data) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(e) => {
            ic_cdk::print(format!("Error: {:?}", e));
            panic!("Failed to restore canister data from stable memory");
        }
    }
}

const DELAY_FOR_REFETCHING_WELL_KNOWN_PRINCIPALS: Duration = Duration::from_secs(1);
fn refetch_well_known_principals() {
    ic_cdk_timers::set_timer(DELAY_FOR_REFETCHING_WELL_KNOWN_PRINCIPALS, || {
        ic_cdk::spawn(update_locally_stored_well_known_principals::update_locally_stored_well_known_principals())
    });
}

const DELAY_FOR_UPGRADING_ALL_INDEXED_USER_CANISTERS: Duration = Duration::from_secs(10);
fn upgrade_all_indexed_user_canisters() {
    ic_cdk_timers::set_timer(DELAY_FOR_UPGRADING_ALL_INDEXED_USER_CANISTERS, || {
        ic_cdk::spawn(update_user_index_upgrade_user_canisters_with_latest_wasm::upgrade_user_canisters_with_latest_wasm())
    });
}

const DELAY_FOR_REMOVING_GLOBAL_ADMIN_AS_CANISTER_CONTROLLER: Duration = Duration::from_secs(5);
fn remove_global_admin_as_controller() {
    ic_cdk_timers::set_timer(
        DELAY_FOR_REMOVING_GLOBAL_ADMIN_AS_CANISTER_CONTROLLER,
        || ic_cdk::spawn(loop_and_remove_global_admin_as_controller()),
    );
}

async fn loop_and_remove_global_admin_as_controller() {
    let user_principal_id_to_canister_id_map = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_canister_id_map
            .clone()
    });

    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow_mut()
            .last_run_upgrade_status
            .successful_upgrade_count = 0;
    });

    for (user_principal_id, user_canister_id) in user_principal_id_to_canister_id_map.iter() {
        let result =
            ic_cdk::api::management_canister::main::update_settings(UpdateSettingsArgument {
                canister_id: *user_canister_id,
                settings: CanisterSettings {
                    controllers: Some(vec![ic_cdk::id()]),
                    ..Default::default()
                },
            })
            .await;

        match result {
            Ok(_) => {
                CANISTER_DATA.with(|canister_data_ref_cell| {
                    canister_data_ref_cell
                        .borrow_mut()
                        .last_run_upgrade_status
                        .successful_upgrade_count += 1;
                });
            }
            Err(e) => {
                CANISTER_DATA.with(|canister_data_ref_cell| {
                    canister_data_ref_cell
                        .borrow_mut()
                        .last_run_upgrade_status
                        .failed_canister_ids
                        .push((*user_principal_id, *user_canister_id, e.1))
                });
            }
        }
    }
}
