use shared_utils::canister_specific::individual_user_template::types::post::PostViewDetailsFromFrontend;

use crate::CANISTER_DATA;

use super::update_scores_and_share_with_post_cache_if_difference_beyond_threshold::update_scores_and_share_with_post_cache_if_difference_beyond_threshold;

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_post_add_view_details(id: u64, details: PostViewDetailsFromFrontend) {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .get(&id)
            .unwrap()
            .clone();

        post_to_update.add_view_details(&details);

        canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .insert(id, post_to_update);
    });

    update_scores_and_share_with_post_cache_if_difference_beyond_threshold(&id);
}
