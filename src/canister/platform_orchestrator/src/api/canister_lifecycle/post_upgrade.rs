use ic_cdk::api::call::ArgDecoderConfig;
use ic_cdk_macros::post_upgrade;
use ic_stable_structures::Memory;
use shared_utils::{
    canister_specific::{platform_orchestrator::types::args::PlatformOrchestratorInitArgs, post_cache},
    common::utils::system_time,
};

use crate::{data_model::memory, CANISTER_DATA};

#[post_upgrade]
pub fn post_upgrade() {
    restore_data_from_stable_memory();
    update_version_from_args();
    initialize_children_merkle();
}

fn restore_data_from_stable_memory() {
    let heap_data = memory::get_upgrades_memory();
    let mut heap_data_len_bytes = [0; 4];
    heap_data.read(0, &mut heap_data_len_bytes);
    let heap_data_len = u32::from_le_bytes(heap_data_len_bytes) as usize;

    let mut canister_data_bytes = vec![0; heap_data_len];
    heap_data.read(4, &mut canister_data_bytes);
    let canister_data =
        minicbor_serde::from_slice(&canister_data_bytes).expect("Failed to deserialize heap data");
    CANISTER_DATA.with_borrow_mut(|cd| {
        *cd = canister_data;
    })
}

fn update_version_from_args() {
    let (upgrade_args,) =
        ic_cdk::api::call::arg_data::<(PlatformOrchestratorInitArgs,)>(ArgDecoderConfig::default());
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.version_detail.version = upgrade_args.version;
        canister_data.version_detail.last_update_on = system_time::get_current_system_time();
    })
}

// TODO: remove this once the upgrade is complete
fn initialize_children_merkle() {
    CANISTER_DATA.with_borrow_mut(|cdata| {
        let mut children: Vec<_> = cdata.subnet_orchestrators().iter().copied().collect();
        children.extend(cdata.post_cache_orchestrators());

        cdata.children_merkle.insert_children(children)
    });
}
