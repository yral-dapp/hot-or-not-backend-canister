use ic_cdk_macros::update;
use shared_utils::common::types::{
    known_principal::KnownPrincipalType, top_posts::post_score_index_item::PostStatus,
};

use crate::CANISTER_DATA;

use super::send_update_post_cache::send_update_post_cache;

#[update]
fn update_post_as_ready_to_view(id: u64) {
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

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .get(&id)
            .unwrap()
            .clone();

        post_to_update.update_status(PostStatus::ReadyToView);

        canister_data_ref_cell
            .borrow_mut()
            .all_created_posts
            .insert(id, post_to_update);
    });

    send_update_post_cache(&id);
}
