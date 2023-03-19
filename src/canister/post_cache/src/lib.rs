use std::cell::RefCell;

use candid::{export_service, Principal};

use data_model::CanisterData;
use shared_utils::{
    access_control::UserAccessRole,
    common::types::{
        init_args::PostCacheInitArgs, known_principal::KnownPrincipalType,
        top_posts::post_score_index_item::PostScoreIndexItem,
    },
    types::canister_specific::post_cache::error_types::TopPostsFetchError,
};

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
