use crate::CANISTER_DATA;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontend;

#[ic_cdk_macros::query]
#[candid::candid_method(query)]
fn get_profile_details() -> UserProfileDetailsForFrontend {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let profile = canister_data_ref_cell.borrow().profile.clone();

        UserProfileDetailsForFrontend {
            principal_id: profile.principal_id.unwrap(),
            display_name: profile.display_name.clone(),
            unique_user_name: profile.unique_user_name.clone(),
            profile_picture_url: profile.profile_picture_url.clone(),
            profile_stats: profile.profile_stats.clone(),
            followers_count: canister_data_ref_cell
                .borrow()
                .principals_that_follow_me
                .len() as u64,
            following_count: canister_data_ref_cell.borrow().principals_i_follow.len() as u64,
        }
    })
}
