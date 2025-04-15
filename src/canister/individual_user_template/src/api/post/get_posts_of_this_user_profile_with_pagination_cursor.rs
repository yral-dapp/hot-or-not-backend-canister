use std::time::SystemTime;

use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        error::GetPostsOfUserProfileError, post::PostDetailsForFrontend,
        profile::UserProfileDetailsForFrontend,
    },
    common::{types::top_posts::post_score_index_item::PostStatus, utils::system_time},
    pagination::{self, PaginationError},
};

use crate::CANISTER_DATA;

#[query]
fn get_posts_of_this_user_profile_with_pagination_cursor(
    from_inclusive_index: u64,
    limit: u64,
) -> Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> {
    let api_caller = ic_cdk::caller();
    let current_time = system_time::get_current_system_time_from_ic();
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.get_posts_with_pagination_cursor(
            from_inclusive_index,
            limit,
            api_caller,
            current_time,
        )
    })
}
mod test {
    use std::{
        collections::{BTreeMap, HashSet},
        time::SystemTime,
    };

    use shared_utils::{
        canister_specific::individual_user_template::types::post::{
            FeedScore, Post, PostViewStatistics,
        },
        common::types::top_posts::post_score_index_item::PostStatus,
    };

    use crate::CanisterData;

    use super::*;

    #[test]
    fn test_get_posts_of_this_user_profile_with_pagination_cursor_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.profile.principal_id = Some(Principal::anonymous());

        let posts = vec![
            Post {
                id: 1,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                hot_or_not_details: None,
                is_nsfw: false,
                slots_left_to_be_computed: (1..=48).collect(),
            },
            Post {
                id: 2,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                hot_or_not_details: None,
                is_nsfw: false,
                slots_left_to_be_computed: (1..=48).collect(),
            },
            Post {
                id: 3,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                hot_or_not_details: None,
                is_nsfw: false,
                slots_left_to_be_computed: (1..=48).collect(),
            },
            Post {
                id: 4,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                hot_or_not_details: None,
                is_nsfw: false,
                slots_left_to_be_computed: (1..=48).collect(),
            },
            Post {
                id: 5,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::BannedDueToUserReporting,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                hot_or_not_details: None,
                is_nsfw: false,
                slots_left_to_be_computed: (1..=48).collect(),
            },
            Post {
                id: 6,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                hot_or_not_details: None,
                is_nsfw: false,
                slots_left_to_be_computed: (1..=48).collect(),
            },
            Post {
                id: 7,
                description: "test post".into(),
                hashtags: Vec::new(),
                video_uid: String::from(""),
                status: PostStatus::Deleted,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
                hot_or_not_details: None,
                is_nsfw: false,
                slots_left_to_be_computed: (1..=48).collect(),
            },
        ];

        posts.into_iter().for_each(|post| {
            let _ = canister_data.add_post(post);
        });
        let api_caller = Principal::anonymous();
        let current_time = SystemTime::now();

        // Test with NSFW filter
        let result = canister_data.get_posts_with_pagination_cursor(0, 3, api_caller, current_time);

        let posts = result.unwrap();
        assert_eq!(posts.len(), 3);
        assert_eq!(posts[0].id, 6);
        assert_eq!(posts[1].id, 4);
        assert_eq!(posts[2].id, 3);

        // Test with NSFW filter
        let result = canister_data.get_posts_with_pagination_cursor(4, 1, api_caller, current_time);

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, 1);
    }
}
