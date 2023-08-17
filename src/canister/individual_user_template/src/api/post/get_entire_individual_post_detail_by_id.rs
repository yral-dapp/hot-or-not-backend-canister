use shared_utils::{
    canister_specific::individual_user_template::types::post::Post,
    common::types::known_principal::KnownPrincipalType,
};

use crate::CANISTER_DATA;

#[ic_cdk::query]
#[candid::candid_method(query)]
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

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let post = canister_data_ref_cell
            .borrow()
            .all_created_posts
            .get(&post_id)
            .unwrap()
            .clone();

        Ok(post)
    })
}
