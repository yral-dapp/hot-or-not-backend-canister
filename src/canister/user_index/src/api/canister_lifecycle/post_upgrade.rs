use candid::Principal;
use ciborium::de;

use ic_cdk::call;
use ic_cdk_macros::post_upgrade;
use ic_stable_structures::Memory;
use shared_utils::{canister_specific::user_index::types::args::UserIndexInitArgs, common::utils::{system_time, task::run_task_concurrently}};
use shared_utils::common::types::known_principal::KnownPrincipalType::CanisterIdPlatformOrchestrator;

use crate::{
    data_model::{canister_upgrade::UpgradeStatus, memory},
    CANISTER_DATA,
};

#[post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
    update_version_from_args();
    insert_platform_orchestrator_in_well_known_principal();
}

fn insert_platform_orchestrator_in_well_known_principal() {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.configuration.known_principal_ids.insert(CanisterIdPlatformOrchestrator, Principal::from_text("74zq4-iqaaa-aaaam-ab53a-cai").unwrap());
    })
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
   let heap_data = memory::get_upgrades_memory();
   let mut heap_data_len_bytes = [0; 4];
   heap_data.read(0, &mut heap_data_len_bytes);
   let heap_data_len = u32::from_le_bytes(heap_data_len_bytes) as usize;

   let mut canister_data_bytes = vec![0; heap_data_len];
   heap_data.read(4, &mut canister_data_bytes);
   let canister_data = de::from_reader(&*canister_data_bytes).expect("Failed to deserialize heap data");
   CANISTER_DATA.with_borrow_mut(|cd| {
        *cd = canister_data;
   })
}