use std::time::Duration;

use shared_utils::{
    canister_specific::user_index::types::args::UserIndexInitArgs,
    common::utils::{stable_memory_serializer_deserializer, system_time},
};

use crate::{
    api::{
        upgrade_individual_user_template::update_user_index_upgrade_user_canisters_with_latest_wasm,
        well_known_principal::update_locally_stored_well_known_principals,
    },
    data_model::{canister_upgrade::UpgradeStatus, configuration::Configuration, CanisterData},
    CANISTER_DATA,
};

use super::pre_upgrade::BUFFER_SIZE_BYTES;

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    update_version_from_args();
    upgrade_all_indexed_user_canisters();
}

fn update_version_from_args() {
    let (upgrade_args,) = ic_cdk::api::call::arg_data::<(UserIndexInitArgs,)>();
    CANISTER_DATA.with(|canister_data_ref| {
        let last_upgrade_status = canister_data_ref.borrow().last_run_upgrade_status.clone();
        let upgrade_status = UpgradeStatus {
            last_run_on: system_time::get_current_system_time_from_ic(),
            failed_canister_ids: vec![],
            version_number: last_upgrade_status.version_number,
            successful_upgrade_count: 0,
            version: upgrade_args.version,
        };
        canister_data_ref.borrow_mut().last_run_upgrade_status = upgrade_status;
    })
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
