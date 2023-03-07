use shared_utils::common::utils::system_time;

use crate::CANISTER_DATA;

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_post_increment_share_count(id: u64) -> u64 {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .get(&id)
            .unwrap()
            .clone();

        let updated_share_count =
            post_to_update.increment_share_count(&system_time::get_current_system_time_from_ic);

        canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .insert(id, post_to_update);

        updated_share_count
    })
}
