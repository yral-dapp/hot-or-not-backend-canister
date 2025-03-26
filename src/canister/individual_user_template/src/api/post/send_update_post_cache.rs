use std::time::SystemTime;

use candid::Principal;
use ic_cdk::api::call;
use shared_utils::common::{
    types::{
        known_principal::KnownPrincipalType, top_posts::post_score_index_item::PostScoreIndexItemV1,
    },
    utils::system_time,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[deprecated]
pub fn send_update_post_cache(post_id: &u64) {
    let current_time = system_time::get_current_system_time();
    let canisters_own_principal_id = ic_cdk::id();

    let (home_feed_index_score_item, hot_or_not_index_score_item): (
        Option<PostScoreIndexItemV1>,
        Option<PostScoreIndexItemV1>,
    ) = CANISTER_DATA.with(|canister_data_ref_cell| {
        update_local_cache_get_items(
            &mut canister_data_ref_cell.borrow_mut(),
            *post_id,
            current_time,
            canisters_own_principal_id,
        )
    });

    let post_cache_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdPostCache)
            .cloned()
            .unwrap()
    });

    if home_feed_index_score_item.is_some() {
        let _ = call::notify(
            post_cache_canister_principal_id,
            "update_post_home_feed",
            (home_feed_index_score_item.unwrap(),),
        );
    }

    if hot_or_not_index_score_item.is_some() {
        let _ = call::notify(
            post_cache_canister_principal_id,
            "update_post_hot_or_not_feed",
            (hot_or_not_index_score_item.clone().unwrap(),),
        );
    }

    if hot_or_not_index_score_item.is_some() {
        let _ = call::notify(
            post_cache_canister_principal_id,
            "update_post_yral_feed",
            (hot_or_not_index_score_item.unwrap(),),
        );
    }
}
