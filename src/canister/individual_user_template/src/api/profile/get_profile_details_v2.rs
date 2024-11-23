use crate::CANISTER_DATA;
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontendV2;

#[query]
pub fn get_profile_details_v2() -> UserProfileDetailsForFrontendV2 {
    CANISTER_DATA.with_borrow(|canister_data_ref_cell| {
        let profile = canister_data_ref_cell.profile.clone();
        let token_balance = &canister_data_ref_cell.my_token_balance;

        UserProfileDetailsForFrontendV2 {
            principal_id: profile.principal_id.unwrap(),
            display_name: profile.display_name.clone(),
            unique_user_name: profile.unique_user_name.clone(),
            profile_picture_url: profile.profile_picture_url.clone(),
            profile_stats: profile.profile_stats,
            followers_count: canister_data_ref_cell.follow_data.follower.len() as u64,
            following_count: canister_data_ref_cell.follow_data.following.len() as u64,
            lifetime_earnings: token_balance.lifetime_earnings,
            referrer_details: profile.referrer_details,
            migration_info: canister_data_ref_cell.migration_info,
        }
    })
}
