use crate::CANISTER_DATA;

use super::update_scores_and_share_with_post_cache_if_difference_beyond_threshold::update_scores_and_share_with_post_cache_if_difference_beyond_threshold;

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_post_increment_share_count(id: u64) -> u64 {
    let response = CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .get(&id)
            .unwrap()
            .clone();

        let updated_share_count = post_to_update.increment_share_count();

        canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .insert(id, post_to_update);

        updated_share_count
    });

    update_scores_and_share_with_post_cache_if_difference_beyond_threshold(&id);

    response
}
