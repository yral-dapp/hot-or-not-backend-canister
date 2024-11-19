use ic_cdk_macros::update;
use shared_utils::common::types::{
    known_principal::KnownPrincipalType, top_posts::post_score_index_item::PostStatus,
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    util::cycles::recharge_canister, CANISTER_DATA,
};

use super::send_update_post_cache::send_update_post_cache;

#[update]
fn update_post_status(id: u64, status: PostStatus) {
    recharge_canister();

    let api_caller = ic_cdk::caller();

    let global_super_admin_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .cloned()
            .unwrap()
    });

    if api_caller != global_super_admin_principal_id {
        return;
    }

    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .get(&id)
            .unwrap()
            .clone();

        post_to_update.update_status(status);

        canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .insert(id, post_to_update);
    });

    send_update_post_cache(&id);
}
