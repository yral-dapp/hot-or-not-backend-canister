use std::time::{Duration, SystemTime};

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
        add_post_to_memory(
            &mut canister_data_ref_cell.borrow_mut(),
            &post_details,
            &system_time::get_current_system_time_from_ic(),
        )
    });

    let post_id = response.clone().unwrap();

    if response.is_ok() {
        update_scores_and_share_with_post_cache_if_difference_beyond_threshold(&post_id);
    }

    // TODO: enable this once rest of the logic implemented
    // if post_details.creator_consent_for_inclusion_in_hot_or_not {
    //     // * schedule hot_or_not outcome tabulation for the 48 hours after the post is created
    //     (1..=48).for_each(|slot_number: u8| {
    //         ic_cdk_timers::set_timer(
    //             Duration::from_secs(slot_number as u64 * 60 * 60),
    //             move || {
    //                 CANISTER_DATA.with(|canister_data_ref_cell| {
    //                     tabulate_hot_or_not_outcome_for_post_slot(
    //                         &mut canister_data_ref_cell.borrow_mut(),
    //                         post_id,
    //                         slot_number,
    //                     );
    //                 });
    //             },
    //         );
    //     })
    // }

    Ok(post_id)
}

fn add_post_to_memory(
    canister_data: &mut CanisterData,
    post_details: &PostDetailsFromFrontend,
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

fn tabulate_hot_or_not_outcome_for_post_slot(
    canister_data: &mut CanisterData,
    post_id: u64,
    slot_id: u8,
) {
    let post_to_tabulate_results_for = canister_data.all_created_posts.get_mut(&post_id).unwrap();
    post_to_tabulate_results_for.tabulate_hot_or_not_outcome_for_slot(&slot_id);
}
