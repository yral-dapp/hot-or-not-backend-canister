use crate::CANISTER_DATA;
use candid::CandidType;
use shared_utils::canister_specific::individual_user_template::types::profile::{
    UserProfileDetailsForFrontend, UserProfileUpdateDetailsFromFrontend,
};

#[derive(CandidType)]
pub enum UpdateProfileDetailsError {
    NotAuthorized,
}

/// # Access Control
/// Only the user whose profile details are stored in this canister can update their details.
#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_profile_display_details(
    user_profile_details: UserProfileUpdateDetailsFromFrontend,
) -> Result<UserProfileDetailsForFrontend, UpdateProfileDetailsError> {
    // * access control
    let current_caller = ic_cdk::caller();
    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id)
        .unwrap();

    if current_caller != my_principal_id {
        return Err(UpdateProfileDetailsError::NotAuthorized);
    }

    Ok(CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut profile = canister_data_ref_cell.borrow_mut().profile.clone();

        profile.display_name = user_profile_details.display_name;
        profile.profile_picture_url = user_profile_details.profile_picture_url;

        canister_data_ref_cell.borrow_mut().profile = profile.clone();

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
    }))
}
