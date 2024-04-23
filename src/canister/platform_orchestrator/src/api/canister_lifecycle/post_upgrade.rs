use std::time::Duration;

use candid::Principal;
use ciborium::de;
use ic_cdk_macros::post_upgrade;
use ic_stable_structures::Memory;
use shared_utils::{
    canister_specific::platform_orchestrator::types::args::PlatformOrchestratorInitArgs,
    common::utils::system_time, constant::CANISTER_RECYCLING_FREQUENCY,
};

use crate::{data_model::memory, CANISTER_DATA};

#[post_upgrade]
pub fn post_upgrade() {
    restore_data_from_stable_memory();
    update_version_from_args();
    start_recycle_canisters_periodic_job();
}

fn restore_data_from_stable_memory() {
    let heap_data = memory::get_upgrades_memory();
    let mut heap_data_len_bytes = [0; 4];
    heap_data.read(0, &mut heap_data_len_bytes);
    let heap_data_len = u32::from_le_bytes(heap_data_len_bytes) as usize;

    let mut canister_data_bytes = vec![0; heap_data_len];
    heap_data.read(4, &mut canister_data_bytes);
    let canister_data =
        de::from_reader(&*canister_data_bytes).expect("Failed to deserialize heap data");
    CANISTER_DATA.with_borrow_mut(|cd| {
        *cd = canister_data;
    })
}

fn update_version_from_args() {
    let (upgrade_args,) = ic_cdk::api::call::arg_data::<(PlatformOrchestratorInitArgs,)>();
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.version_detail.version = upgrade_args.version;
        canister_data.version_detail.last_update_on = system_time::get_current_system_time();
    })
}

fn start_recycle_canisters_periodic_job() {
    ic_cdk_timers::set_timer_interval(Duration::from_secs(CANISTER_RECYCLING_FREQUENCY), || {
        ic_cdk::spawn(async {
            // iterate through all_subnet_orchestrator_canisters_list
            let mut subnet_orchestrators = CANISTER_DATA.with(|canister_data| {
                let canister_data = canister_data.borrow();
                canister_data
                    .all_subnet_orchestrator_canisters_list
                    .iter()
                    .map(|canister_id| canister_id.clone())
                    .collect::<Vec<Principal>>()
            });

            // remove rimrc-piaaa-aaaao-aaljq-cai from subnet_orchestrators
            subnet_orchestrators.retain(|canister_id| {
                *canister_id != Principal::from_text("rimrc-piaaa-aaaao-aaljq-cai").unwrap()
            });

            // iter each orchestrator canister and call recycle_canisters
            for orchestrator_canister_id in subnet_orchestrators {
                // second check just in case
                if orchestrator_canister_id
                    == Principal::from_text("rimrc-piaaa-aaaao-aaljq-cai").unwrap()
                {
                    continue;
                }

                let res: ((),) = ic_cdk::call(orchestrator_canister_id, "recycle_canisters", ())
                    .await
                    .expect("Failed to call recycle_canisters");
            }
        });
    });
}
