use ic_cdk_macros::update;

use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA};

#[update]
fn update_post_increment_share_count(id: u64) -> u64 {
    notify_to_recharge_canister();

    let response = CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .get_post(&id)
            .unwrap()
            .clone();

        let updated_share_count = post_to_update.increment_share_count();

        canister_data_ref_cell.borrow_mut().add_post(post_to_update);

        updated_share_count
    });

    response
}
