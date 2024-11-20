use ic_cdk_macros::update;

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    util::cycles::notify_to_recharge_canister, CANISTER_DATA,
};

use super::update_scores_and_share_with_post_cache_if_difference_beyond_threshold::update_scores_and_share_with_post_cache_if_difference_beyond_threshold;

#[update]
fn update_post_toggle_like_status_by_caller(id: u64) -> bool {
    notify_to_recharge_canister();
    update_last_canister_functionality_access_time();

    let caller_id = ic_cdk::caller();

    let response = CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .get(&id)
            .unwrap()
            .clone();

        let updated_like_status = post_to_update.toggle_like_status(&caller_id);

        canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .insert(id, post_to_update);

        updated_like_status
    });

    update_scores_and_share_with_post_cache_if_difference_beyond_threshold(&id);

    response
}
