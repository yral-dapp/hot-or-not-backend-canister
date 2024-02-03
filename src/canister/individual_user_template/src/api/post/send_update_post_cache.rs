use ic_cdk::api::call;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

pub fn send_update_post_cache(post_id: &u64) {
    let post_item = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_created_posts
            .get(&post_id)
            .cloned()
            .unwrap()
    });

    let post_cache_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdPostCache)
            .cloned()
            .unwrap()
    });

    let _ = call::notify(
        post_cache_canister_principal_id,
        "update_post_home_feed",
        (post_item.clone(),),
    );

    let _ = call::notify(
        post_cache_canister_principal_id,
        "update_post_hot_or_not_feed",
        (post_item,),
    );
}
