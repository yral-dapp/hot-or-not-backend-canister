use std:: time::Duration;
use candid::Principal;
use ic_cdk::notify;
use shared_utils::common::{
    types::top_posts::{post_score_index_item::PostScoreIndexItemV1, LATEST_POSTS_WINDOW},
    utils::system_time::get_current_system_time,
};

use crate::CANISTER_DATA;

const TRIGGER_UPDATE_HOT_OR_NOT_INDEX: Duration = Duration::from_secs(60 * 60);
const TRIGGER_RECONCILE_SCORES: Duration = Duration::from_secs(60 * 60 * 5);
const RECONCILE_SCORES_UPTO: usize = 100;

pub fn trigger_update_hot_or_not_index() {
    let last_updated_hot_or_not_timestamp_index = CANISTER_DATA.with(|canister_data| {
        canister_data
            .borrow()
            .metadata
            .last_updated_hot_or_not_timestamp_index
    });

    let now = get_current_system_time();
    if now
        .duration_since(
            last_updated_hot_or_not_timestamp_index
                .unwrap_or_else(|| now - TRIGGER_UPDATE_HOT_OR_NOT_INDEX),
        )
        .unwrap_or_default()
        >= TRIGGER_UPDATE_HOT_OR_NOT_INDEX
    {
        // Update the hot or not index

        let old_post_ids = CANISTER_DATA.with(|canister_data| {
            let canister_data = canister_data.borrow();

            canister_data
                .posts_index_sorted_by_hot_or_not_feed_score_v1
                .item_time_index
                .iter()
                .take_while(|(&created_at, _)| {
                    now.duration_since(created_at).unwrap_or_default() >= LATEST_POSTS_WINDOW
                })
                .map(|(_, post_ids)| post_ids)
                .flatten()
                .cloned()
                .map(|post_id| {
                    canister_data
                        .posts_index_sorted_by_hot_or_not_feed_score_v1
                        .item_presence_index
                        .get(&post_id)
                        .unwrap()
                        .clone()
                })
                .collect::<Vec<PostScoreIndexItemV1>>()
        });

        CANISTER_DATA.with(|canister_data| {
            let mut canister_data = canister_data.borrow_mut();
            let hot_or_not_index =
                &mut canister_data.posts_index_sorted_by_hot_or_not_feed_score_v1;
            // Replace the old post ids
            for post in old_post_ids {
                hot_or_not_index.replace(&post);
            }

            canister_data
                .metadata
                .last_updated_hot_or_not_timestamp_index = Some(now);
        });
    }
}

// TODO: Add integration tests
pub fn trigger_reconcile_scores() {
    let last_updated_reconcile_scores = CANISTER_DATA.with(|canister_data| {
        canister_data
            .borrow()
            .metadata
            .last_updated_reconcile_scores
    });

    let now = get_current_system_time();
    if now
        .duration_since(
            last_updated_reconcile_scores.unwrap_or_else(|| now - TRIGGER_UPDATE_HOT_OR_NOT_INDEX),
        )
        .unwrap_or_default()
        >= TRIGGER_RECONCILE_SCORES
    {
        // Reconcile home feed scores
        //
        let top_home_feed = CANISTER_DATA.with(|canister_data| {
            canister_data
                .borrow()
                .posts_index_sorted_by_home_feed_score_v1
                .iter()
                .take(RECONCILE_SCORES_UPTO)
                .map(|post| (post.publisher_canister_id.clone(), post.post_id))
                .collect::<Vec<(Principal, u64)>>()
        });
        // Change (Principal, u64) to HashMap with Principal as key and Vec<u64> as value
        let mut top_home_feed_by_user = std::collections::HashMap::new();
        for (principal, post_id) in top_home_feed {
            top_home_feed_by_user
                .entry(principal)
                .or_insert_with(Vec::new)
                .push(post_id);
        }

        // cdk::api::call::notify principal with posts
        for (principal, post_ids) in top_home_feed_by_user {
            // cdk::api::call::notify(principal, post_ids);
            let _ = notify(
                principal,
                "check_and_update_scores_and_share_with_post_cache_if_difference_beyond_threshold",
                (post_ids,),
            );
        }

        // Reconcile hot or not feed scores
        //
        let top_hot_or_not_feed = CANISTER_DATA.with(|canister_data| {
            canister_data
                .borrow()
                .posts_index_sorted_by_hot_or_not_feed_score_v1
                .iter()
                .take(RECONCILE_SCORES_UPTO)
                .map(|post| (post.publisher_canister_id.clone(), post.post_id))
                .collect::<Vec<(Principal, u64)>>()
        });
        // Change (Principal, u64) to HashMap with Principal as key and Vec<u64> as value
        let mut top_hot_or_not_feed_by_user = std::collections::HashMap::new();
        for (principal, post_id) in top_hot_or_not_feed {
            top_hot_or_not_feed_by_user
                .entry(principal)
                .or_insert_with(Vec::new)
                .push(post_id);
        }

        // cdk::api::call::notify principal with posts
        for (principal, post_ids) in top_hot_or_not_feed_by_user {
            // cdk::api::call::notify(principal, post_ids);
            let _ = notify(
                principal,
                "check_and_update_scores_and_share_with_post_cache_if_difference_beyond_threshold",
                (post_ids,),
            );
        }

        CANISTER_DATA.with(|canister_data| {
            let mut canister_data = canister_data.borrow_mut();
            canister_data.metadata.last_updated_reconcile_scores = Some(now);
        });
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {

    use candid::Principal;
    use shared_utils::common::{
        types::top_posts::post_score_index_item::{PostScoreIndexItemV1, PostStatus},
        utils::system_time::set_mock_time,
    };

    use super::*;

    #[test]
    fn test_trigger_update_hot_or_not_index() {
        let canister_data = CanisterData::default();
        CANISTER_DATA.with(|data| {
            *data.borrow_mut() = canister_data;
        });

        let created_at_now = get_current_system_time();
        let creted_at_earlier = created_at_now - (LATEST_POSTS_WINDOW - Duration::from_secs(5));

        let posts = vec![
            PostScoreIndexItemV1 {
                score: 1,
                post_id: 1,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_now),
            },
            PostScoreIndexItemV1 {
                score: 2,
                post_id: 2,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_now),
            },
            PostScoreIndexItemV1 {
                score: 3,
                post_id: 3,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(creted_at_earlier),
            },
            PostScoreIndexItemV1 {
                score: 4,
                post_id: 4,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(creted_at_earlier),
            },
            PostScoreIndexItemV1 {
                score: 5,
                post_id: 5,
                publisher_canister_id: Principal::from_text("w4nuc-waaaa-aaaao-aal2a-cai").unwrap(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(creted_at_earlier),
            },
        ];

        // Replace in CanisterData
        CANISTER_DATA.with(|canister_data| {
            let mut canister_data = canister_data.borrow_mut();
            let hot_or_not_index =
                &mut canister_data.posts_index_sorted_by_hot_or_not_feed_score_v1;
            for post in posts {
                hot_or_not_index.replace(&post);
            }
        });

        // Check if the posts are in the right order in CANISTER
        let post_score_index = CANISTER_DATA.with(|canister_data| {
            let canister_data = canister_data.borrow();
            canister_data
                .posts_index_sorted_by_hot_or_not_feed_score_v1
                .clone()
        });
        let mut post_score_index_iter = post_score_index.iter();
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 5);
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 4);
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 3);
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 2);
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 1);
        assert_eq!(post_score_index_iter.next(), None);

        // MockClock::advance_system_time(Duration::from_secs(10));
        set_mock_time(created_at_now + Duration::from_secs(10));

        trigger_update_hot_or_not_index();

        let post_score_index = CANISTER_DATA.with(|canister_data| {
            let canister_data = canister_data.borrow();
            canister_data
                .posts_index_sorted_by_hot_or_not_feed_score_v1
                .clone()
        });
        let mut post_score_index_iter = post_score_index.iter();
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 2);
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 1);
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 5);
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 4);
        assert_eq!(post_score_index_iter.next().unwrap().post_id, 3);
        assert_eq!(post_score_index_iter.next(), None);
    }
}
