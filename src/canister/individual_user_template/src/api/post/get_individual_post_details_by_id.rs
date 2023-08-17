use shared_utils::{
    canister_specific::individual_user_template::types::{
        post::PostDetailsForFrontend, profile::UserProfileDetailsForFrontend,
    },
    common::utils::system_time,
};

use crate::CANISTER_DATA;

#[ic_cdk::query]
#[candid::candid_method(query)]
pub fn get_individual_post_details_by_id(post_id: u64) -> PostDetailsForFrontend {
    let api_caller = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let post = canister_data_ref_cell
            .borrow()
            .all_created_posts
            .get(&post_id)
            .unwrap()
            .clone();
        let profile = &canister_data_ref_cell.borrow().profile;
        let followers = &canister_data_ref_cell.borrow().principals_that_follow_me;
        let following = &canister_data_ref_cell.borrow().principals_i_follow;
        let token_balance = &canister_data_ref_cell.borrow().my_token_balance;

        post.get_post_details_for_frontend_for_this_post(
            UserProfileDetailsForFrontend {
                display_name: profile.display_name.clone(),
                followers_count: followers.len() as u64,
                following_count: following.len() as u64,
                principal_id: profile.principal_id.unwrap(),
                profile_picture_url: profile.profile_picture_url.clone(),
                profile_stats: profile.profile_stats,
                unique_user_name: profile.unique_user_name.clone(),
                lifetime_earnings: token_balance.lifetime_earnings,
            },
            api_caller,
            &system_time::get_current_system_time_from_ic(),
        )
    })
}
