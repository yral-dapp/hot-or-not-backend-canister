use ic_cdk_macros::query;
use shared_utils::{
    canister_specific::individual_user_template::types::post::Post,
    common::types::known_principal::KnownPrincipalType,
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[query]
pub fn get_entire_individual_post_detail_by_id(post_id: u64) -> Result<Post, ()> {
    let api_caller = ic_cdk::caller();

    let super_admin_user = CANISTER_DATA.with(|canister_data_ref_cell| {
        *canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .unwrap()
    });

    if api_caller != super_admin_user {
        return Err(());
    }

    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let post = canister_data_ref_cell
            .borrow()
            .get_post(&post_id)
            .unwrap()
            .clone();

        Ok(post)
    })
}
