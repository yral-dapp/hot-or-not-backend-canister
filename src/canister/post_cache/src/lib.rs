use std::cell::RefCell;

use candid::Principal;

use data_model::CanisterData;
use ic_cdk_macros::export_candid;
use shared_utils::{
    canister_specific::post_cache::types::arg::PostCacheInitArgs,
    common::types::{
        known_principal::KnownPrincipalType,
        top_posts::post_score_index_item::{PostScoreIndexItem, PostScoreIndexItemV1, PostStatus},
    },
    types::canister_specific::post_cache::error_types::TopPostsFetchError,
};

mod api;
mod data_model;
mod util;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();