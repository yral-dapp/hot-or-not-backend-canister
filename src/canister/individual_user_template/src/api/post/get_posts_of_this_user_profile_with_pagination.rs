use shared_utils::{
    canister_specific::individual_user_template::types::error::GetPostsOfUserProfileError,
    pagination::{self, PaginationError},
    types::canister_specific::individual_user_template::post::PostDetailsForFrontend,
};

use crate::CANISTER_DATA;

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_posts_of_this_user_profile_with_pagination(
    from_inclusive_id: u64,
    to_exclusive_id: u64,
) -> Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> {
    let (from_inclusive_id, to_exclusive_id) = pagination::get_pagination_bounds(
        from_inclusive_id,
        to_exclusive_id,
        CANISTER_DATA
            .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().all_created_posts.len())
            as u64,
    )
    .map_err(|e| match e {
        PaginationError::InvalidBoundsPassed => GetPostsOfUserProfileError::InvalidBoundsPassed,
        PaginationError::ReachedEndOfItemsList => GetPostsOfUserProfileError::ReachedEndOfItemsList,
        PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest => {
            GetPostsOfUserProfileError::ExceededMaxNumberOfItemsAllowedInOneRequest
        }
    })?;

    let api_caller = ic_cdk::caller();

    Ok((from_inclusive_id..to_exclusive_id)
        .map(|id| {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                let post = canister_data_ref_cell
                    .borrow()
                    .all_created_posts
                    .get(&id)
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
                    liked_by_me: post.likes.contains(&api_caller),
                    home_feed_ranking_score: post.homefeed_ranking_score,
                    hot_or_not_feed_ranking_score: post
                        .hot_or_not_details
                        .as_ref()
                        .map(|details| details.score),
                }
            })
        })
        .collect())
}
