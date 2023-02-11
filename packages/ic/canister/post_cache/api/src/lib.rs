use std::cell::RefCell;

use candid::{export_service, Principal};
use ic_cdk::storage;

use ic_stable_memory::utils::ic_types::SPrincipal;
use post_cache_lib::{access_control, CanisterData};
use shared_utils::{
    access_control::UserAccessRole,
    common::types::init_args::PostCacheInitArgs,
    types::{
        canister_specific::post_cache::error_types::TopPostsFetchError,
        top_posts::v0::PostScoreIndexItem,
    },
};

mod api;
#[cfg(test)]
mod test;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

#[ic_cdk_macros::init]
#[candid::candid_method(init)]
fn init(init_args: PostCacheInitArgs) {
    // TODO: populate the canister data access control map
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        access_control::setup_initial_access_control_v1(
            &mut canister_data.access_control_map,
            &init_args.known_principal_ids,
        );

        canister_data.my_known_principal_ids_map = init_args
            .known_principal_ids
            .iter()
            .map(|(k, v)| (k.clone(), SPrincipal(v.clone())))
            .collect();
    });
}

#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade() {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.take();

        storage::stable_save((canister_data,)).ok();
    });
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok((canister_data,)) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(_) => {
            panic!("Failed to restore canister data from stable memory");
        }
    }
}

#[ic_cdk_macros::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}
