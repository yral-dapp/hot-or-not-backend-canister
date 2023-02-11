use shared_utils::{
    canister_specific::individual_user_template::types::post::v1::Post, date_time::system_time,
    types::post::PostDetailsFromFrontend,
};

use crate::CANISTER_DATA;

/// # Access Control
/// Only the user whose profile details are stored in this canister can create a post.
#[ic_cdk_macros::update]
#[candid::candid_method(update)]
fn add_post(post_details: PostDetailsFromFrontend) -> u64 {
    // * access control
    let current_caller = ic_cdk::caller();
    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id);
    assert_eq!(
        my_principal_id,
        Some(current_caller),
        "Only the user whose profile details are stored in this canister can create a post."
    );

    let id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().all_created_posts.len())
        as u64;

    let mut post = Post::new(
        id,
        post_details,
        &system_time::get_current_system_time_from_ic,
    );

    post.recalculate_home_feed_score(&system_time::get_current_system_time_from_ic);
    post.recalculate_hot_or_not_feed_score(&system_time::get_current_system_time_from_ic);

    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .insert(id, post)
    });

    id
}
