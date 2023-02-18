use ic_stable_memory::utils::ic_types::SPrincipal;
use shared_utils::types::canister_specific::individual_user_template::post::PostDetailsForFrontend;

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

        PostDetailsForFrontend {
            id: post.id,
            created_by_display_name: profile.display_name.clone(),
            created_by_unique_user_name: profile.unique_user_name.clone(),
            created_by_user_principal_id: profile.principal_id.unwrap(),
            created_by_profile_photo_url: profile.profile_picture_url.clone(),
            created_at: post.created_at,
            description: post.description.clone(),
            hashtags: post.hashtags.clone(),
            video_uid: post.video_uid.clone(),
            status: post.status.clone(),
            total_view_count: post.view_stats.total_view_count,
            like_count: post.likes.len() as u64,
            liked_by_me: post.likes.contains(&SPrincipal(api_caller)),
            home_feed_ranking_score: post.homefeed_ranking_score,
            hot_or_not_feed_ranking_score: post
                .hot_or_not_feed_details
                .as_ref()
                .map(|details| details.score),
        }
    })
}
