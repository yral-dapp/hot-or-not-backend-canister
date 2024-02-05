use std::cell::RefCell;

use candid::{export_service, Principal};

use data_model::CanisterData;
use shared_utils::{
    canister_specific::post_cache::types::arg::PostCacheInitArgs,
    common::types::{
        known_principal::KnownPrincipalType, top_posts::post_score_index_item::PostScoreIndexItem,
    },
    types::canister_specific::post_cache::error_types::TopPostsFetchError,
};

//TODO: add method get_cycle_balance in post_cache canister.

mod api;
mod data_model;
#[cfg(test)]
mod test;
mod util;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}
