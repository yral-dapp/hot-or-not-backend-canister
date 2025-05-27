use std::time::Duration;

use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::post::PostDetailsFromFrontend,
    common::utils::system_time,
};

use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA};

/// #### Access Control
/// Only the user whose profile details are stored in this canister can create a post.
#[update]
fn add_post_v2(post_details: PostDetailsFromFrontend) -> Result<u64, String> {
    notify_to_recharge_canister();

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

    let response = CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.add_post_to_memory(
            &post_details,
            &system_time::get_current_system_time_from_ic(),
        )
    });

    let post_id = response;

    Ok(post_id)
}
