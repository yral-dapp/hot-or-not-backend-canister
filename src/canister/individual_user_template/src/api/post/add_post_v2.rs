use std::time::{Duration, SystemTime};

use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::post::{Post, PostDetailsFromFrontend},
    common::utils::system_time,
};

use crate::{
    api::{
        canister_management::update_last_access_time::update_last_canister_functionality_access_time,
        hot_or_not_bet::tabulate_hot_or_not_outcome_for_post_slot::{
            tabulate_hot_or_not_outcome_for_post_slot, tabulate_hot_or_not_outcome_for_post_slot_v1,
        },
    },
    data_model::CanisterData,
    util::cycles::notify_to_recharge_canister,
    CANISTER_DATA,
};

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

    update_last_canister_functionality_access_time();

    let response = CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.add_post_to_memory(
            &post_details,
            &system_time::get_current_system_time_from_ic(),
        )
    });

    let post_id = response;

    (1..=48).for_each(|slot_number: u8| {
        ic_cdk_timers::set_timer(
            Duration::from_secs(slot_number as u64 * 60 * 60),
            move || {
                ic_cdk::spawn(tabulate_hot_or_not_outcome_for_post_slot(
                    post_id,
                    slot_number,
                ));
                ic_cdk::spawn(tabulate_hot_or_not_outcome_for_post_slot_v1(
                    post_id,
                    slot_number,
                ));
            },
        );
    });

    Ok(post_id)
}
