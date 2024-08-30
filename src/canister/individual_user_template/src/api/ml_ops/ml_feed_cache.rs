use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};
use ic_cdk_macros::{query, update};
use shared_utils::{
    canister_specific::individual_user_template::types::ml_data::MLFeedCacheItem,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_ml_feed_cache(ml_feed_cache_items: Vec<MLFeedCacheItem>) -> Result<String, String> {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data| {
        let mut canister_data = canister_data.borrow_mut();

        // insert ml_feed_cache_items into canister_data.ml_feed_cache at the start of Vec
        canister_data
            .ml_feed_cache
            .splice(0..0, ml_feed_cache_items);

        // drain ml_feed_cache from the end if it exceeds the limit 100
        if canister_data.ml_feed_cache.len() > 200 {
            canister_data.ml_feed_cache.drain(200..);
        }
    });

    Ok("Success".into())
}

#[query]
fn get_ml_feed_cache_paginated(start_index: usize, count: usize) -> Vec<MLFeedCacheItem> {
    CANISTER_DATA.with(|canister_data| {
        let canister_data = canister_data.borrow();

        let end_index = std::cmp::min(start_index + count, canister_data.ml_feed_cache.len());
        canister_data.ml_feed_cache[start_index..end_index].to_vec()
    })
}
