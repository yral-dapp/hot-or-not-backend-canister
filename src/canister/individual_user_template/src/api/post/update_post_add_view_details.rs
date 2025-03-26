use ic_cdk_macros::update;
use shared_utils::canister_specific::individual_user_template::types::post::PostViewDetailsFromFrontend;

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    util::cycles::notify_to_recharge_canister, CANISTER_DATA,
};

#[update]
fn update_post_add_view_details(id: u64, details: PostViewDetailsFromFrontend) {
    notify_to_recharge_canister();
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .get_post(&id)
            .unwrap()
            .clone();

        post_to_update.add_view_details(&details);

        canister_data_ref_cell.borrow_mut().add_post(post_to_update);
    });
}
