use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, CANISTER_DATA,
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

        update_ml_feed_cache_impl(ml_feed_cache_items, &mut canister_data)
    });

    Ok("Success".into())
}

fn update_ml_feed_cache_impl(
    ml_feed_cache_items: Vec<MLFeedCacheItem>,
    canister_data: &mut CanisterData,
) -> Result<String, String> {
    // insert ml_feed_cache_items into canister_data.ml_feed_cache at the start of Vec
    canister_data
        .ml_feed_cache
        .splice(0..0, ml_feed_cache_items);

    // drain ml_feed_cache from the end if it exceeds the limit 100
    if canister_data.ml_feed_cache.len() > 200 {
        canister_data.ml_feed_cache.drain(200..);
    }

    Ok("Success".into())
}

#[query]
fn get_ml_feed_cache_paginated(start_index: usize, count: usize) -> Vec<MLFeedCacheItem> {
    CANISTER_DATA.with(|canister_data| {
        let canister_data = canister_data.borrow();

        get_ml_feed_cache_paginated_impl(start_index, count, &canister_data)
    })
}

fn get_ml_feed_cache_paginated_impl(
    start_index: usize,
    count: usize,
    canister_data: &CanisterData,
) -> Vec<MLFeedCacheItem> {
    let end_index = std::cmp::min(start_index + count, canister_data.ml_feed_cache.len());

    if start_index >= end_index {
        return vec![];
    }

    canister_data.ml_feed_cache[start_index..end_index].to_vec()
}

#[cfg(test)]
mod test {
    use candid::Principal;

    use super::*;

    #[test]
    fn test_update_ml_feed_cache() {
        let mut canister_data = CanisterData::default();
        let mut ml_feed_cache_items = vec![];
        for i in 0..200 {
            ml_feed_cache_items.push(MLFeedCacheItem {
                post_id: i,
                canister_id: Principal::anonymous(),
                video_id: "dafds".to_string(),
                creator_principal_id: None,
            });
        }

        let result = update_ml_feed_cache_impl(ml_feed_cache_items, &mut canister_data);

        assert_eq!(result, Ok("Success".into()));

        let result = get_ml_feed_cache_paginated_impl(0, 200, &canister_data);

        assert_eq!(result.len(), 200);
        // check if the first item is the same as the first item in ml_feed_cache_items
        assert_eq!(result[0].post_id, 0);
        // check if the last item is the same as the last item in ml_feed_cache_items
        assert_eq!(result[199].post_id, 199);

        // add 100 more items from 200 to 299
        let mut new_ml_feed_cache_items = vec![];
        for i in 200..300 {
            new_ml_feed_cache_items.push(MLFeedCacheItem {
                post_id: i,
                canister_id: Principal::anonymous(),
                video_id: "dafds".to_string(),
                creator_principal_id: None,
            });
        }

        let result = update_ml_feed_cache_impl(new_ml_feed_cache_items, &mut canister_data);

        assert_eq!(result, Ok("Success".into()));

        let result = get_ml_feed_cache_paginated_impl(0, 200, &canister_data);

        assert_eq!(result.len(), 200);
        // check if the first item is the same as the first item in new_ml_feed_cache_items
        assert_eq!(result[0].post_id, 200);
        // check if the last item is the same as the middle item in ml_feed_cache_items
        assert_eq!(result[199].post_id, 99);

        // get the next 100 items
        let result = get_ml_feed_cache_paginated_impl(200, 200, &canister_data);

        assert_eq!(result.len(), 0);

        // get out of bound
        let result = get_ml_feed_cache_paginated_impl(2000, 200, &canister_data);

        assert_eq!(result.len(), 0);
    }
}
