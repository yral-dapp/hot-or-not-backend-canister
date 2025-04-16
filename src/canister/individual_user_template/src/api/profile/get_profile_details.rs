use crate::CANISTER_DATA;
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontend;

#[query]
fn get_profile_details() -> UserProfileDetailsForFrontend {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let profile = canister_data_ref_cell.borrow().profile.clone();
        let token_balance = &canister_data_ref_cell.borrow().my_token_balance;

        UserProfileDetailsForFrontend {
            principal_id: profile.principal_id.unwrap(),
            display_name: None,
            unique_user_name: None,
            profile_picture_url: profile.profile_picture_url.clone(),
            profile_stats: profile.profile_stats,
            followers_count: 0,
            following_count: 0,
            lifetime_earnings: token_balance.lifetime_earnings,
            referrer_details: profile.referrer_details,
        }
    })
}
