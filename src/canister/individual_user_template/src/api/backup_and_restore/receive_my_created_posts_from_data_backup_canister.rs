use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::post::Post,
    common::types::known_principal::KnownPrincipalType,
};

use crate::CANISTER_DATA;

#[update]
fn receive_my_created_posts_from_data_backup_canister(all_posts_chunk_vec: Vec<Post>) {
    let caller = ic_cdk::caller();
    let data_backup_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdDataBackup)
            .cloned()
            .unwrap()
    });

    if caller != data_backup_canister_id {
        return;
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        for post in all_posts_chunk_vec {
            canister_data.all_created_posts.insert(post.id, post);
        }
    });
}
