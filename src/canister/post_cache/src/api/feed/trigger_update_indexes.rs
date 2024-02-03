use std::{
    cell::RefCell,
    time::{Duration, SystemTime},
};

use shared_utils::common::types::top_posts::{
    post_score_index_item::PostScoreIndexItemV1, LATEST_POSTS_WINDOW,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

const TRIGGER_UPDATE_HOT_OR_NOT_INDEX: Duration = Duration::from_secs(60 * 60);

fn trigger_update_hot_or_not_index() {
    let last_updated_hot_or_not_timestamp_index = CANISTER_DATA.with(|canister_data| {
        canister_data
            .borrow()
            .metadata
            .last_updated_hot_or_not_timestamp_index
    });

    let now = SystemTime::now();
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

#[cfg(test)]
mod tests {
    use std::thread;

    use candid::Principal;
    use shared_utils::common::types::top_posts::post_score_index_item::{
        PostScoreIndexItemV1, PostStatus,
    };

    use super::*;

    #[test]
    fn test_trigger_update_hot_or_not_index() {
        let canister_data = CanisterData::default();
        CANISTER_DATA.with(|data| {
            *data.borrow_mut() = canister_data;
        });

        let created_at_now = SystemTime::now();
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

        thread::sleep(Duration::from_secs(5));

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
