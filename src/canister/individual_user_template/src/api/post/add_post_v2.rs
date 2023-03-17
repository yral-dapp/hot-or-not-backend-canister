use std::time::SystemTime;

use shared_utils::{
    canister_specific::individual_user_template::types::post::{Post, PostDetailsFromFrontend},
    common::utils::system_time,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

use super::update_scores_and_share_with_post_cache_if_difference_beyond_threshold::update_scores_and_share_with_post_cache_if_difference_beyond_threshold;

/// #### Access Control
/// Only the user whose profile details are stored in this canister can create a post.
#[ic_cdk::update]
#[candid::candid_method(update)]
fn add_post_v2(post_details: PostDetailsFromFrontend) -> Result<u64, String> {
    // * access control
    let current_caller = ic_cdk::caller();
    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id);
    if my_principal_id != Some(current_caller) {
        return Err(
            "Only the user whose profile details are stored in this canister can create a post."
                .to_string(),
        );
    };

    let response = CANISTER_DATA.with(|canister_data_ref_cell| {
        add_post_v2_impl(
            &mut canister_data_ref_cell.borrow_mut(),
            post_details,
            &system_time::get_current_system_time_from_ic(),
        )
    });

    if response.is_ok() {
        update_scores_and_share_with_post_cache_if_difference_beyond_threshold(
            response.clone().unwrap(),
        );
    }

    response
}

fn add_post_v2_impl(
    canister_data: &mut CanisterData,
    post_details: PostDetailsFromFrontend,
    current_system_time: &SystemTime,
) -> Result<u64, String> {
    let new_post = Post::new(
        canister_data.all_created_posts.len() as u64,
        post_details,
        current_system_time,
    );
    let new_post_id = new_post.id;
    canister_data
        .all_created_posts
        .insert(new_post.id, new_post);
    Ok(new_post_id)
}
