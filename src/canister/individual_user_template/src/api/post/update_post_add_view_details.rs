use shared_utils::{
    canister_specific::individual_user_template::types::post::v0::PostViewDetailsFromFrontend,
    date_time::system_time,
};

use crate::CANISTER_DATA;

#[ic_cdk_macros::update]
#[candid::candid_method(update)]
fn update_post_add_view_details(id: u64, details: PostViewDetailsFromFrontend) {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .get(&id)
            .unwrap()
            .clone();

        post_to_update.add_view_details(details, &system_time::get_current_system_time_from_ic);

        canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .insert(id, post_to_update);
    });
}
