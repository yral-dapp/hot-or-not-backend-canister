use ic_cdk_macros::update;
use shared_utils::common::types::top_posts::post_score_index_item::{
    PostScoreIndexItemV1, PostStatus,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[update]
fn update_post_yral_feed(post: PostScoreIndexItemV1) {
    let caller = ic_cdk::caller();
    if post.publisher_canister_id != caller {
        return;
    }

    CANISTER_DATA.with(|canister_data| {
        let mut canister_data = canister_data.borrow_mut();

        update_post_yral_feed_impl(post, &mut canister_data);
    });
}

fn update_post_yral_feed_impl(post: PostScoreIndexItemV1, canister_data: &mut CanisterData) {
    let item_prescence_index = &mut canister_data
        .posts_index_sorted_by_yral_feed_score
        .item_presence_index;

    let global_id = (post.publisher_canister_id, post.post_id);
    if item_prescence_index.get(&global_id).is_some() {
        if post.status == PostStatus::BannedDueToUserReporting {
            canister_data
                .posts_index_sorted_by_yral_feed_score
                .remove(&post);
        } else {
            canister_data
                .posts_index_sorted_by_yral_feed_score
                .replace(&post);
        }
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;
    use shared_utils::common::types::top_posts::post_score_index_item::PostStatus;
    use std::time::{Duration, SystemTime};

    use super::*;

    #[test]
    fn test_update_post_yral_feed_impl() {
        let mut canister_data = CanisterData::default();
        let created_at_now = SystemTime::now();
        let created_at_ealier = created_at_now - Duration::from_secs(48 * 60 * 60 + 1);

        let post_1 = PostScoreIndexItemV1 {
            post_id: 1,
            score: 1,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: false,
            status: PostStatus::ReadyToView,
            created_at: Some(created_at_now),
        };
        let post_2 = PostScoreIndexItemV1 {
            post_id: 2,
            score: 2,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: false,
            status: PostStatus::ReadyToView,
            created_at: Some(created_at_now),
        };

        canister_data
            .posts_index_sorted_by_yral_feed_score
            .replace(&post_1);
        canister_data
            .posts_index_sorted_by_yral_feed_score
            .replace(&post_2);

        let iter_posts = canister_data
            .posts_index_sorted_by_yral_feed_score
            .iter()
            .collect::<Vec<_>>();

        assert_eq!(iter_posts.len(), 2);
        assert_eq!(iter_posts[0], &post_2);
        assert_eq!(iter_posts[1], &post_1);

        let new_post_2 = PostScoreIndexItemV1 {
            post_id: 2,
            score: 10,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: true,
            status: PostStatus::ReadyToView,
            created_at: Some(created_at_ealier),
        };

        update_post_yral_feed_impl(new_post_2.clone(), &mut canister_data);

        let iter_posts = canister_data
            .posts_index_sorted_by_yral_feed_score
            .iter()
            .collect::<Vec<_>>();

        assert_eq!(iter_posts.len(), 2);
        assert_eq!(iter_posts[0], &post_1);
        assert_eq!(iter_posts[1], &new_post_2);
    }

    #[test]
    fn test_update_post_yral_feed_impl_banned() {
        let mut canister_data = CanisterData::default();
        let created_at_now = SystemTime::now();
        let created_at_ealier = created_at_now - Duration::from_secs(48 * 60 * 60 + 1);

        let post_1 = PostScoreIndexItemV1 {
            post_id: 1,
            score: 1,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: false,
            status: PostStatus::ReadyToView,
            created_at: Some(created_at_now),
        };
        let post_2 = PostScoreIndexItemV1 {
            post_id: 2,
            score: 2,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: false,
            status: PostStatus::ReadyToView,
            created_at: Some(created_at_now),
        };

        canister_data
            .posts_index_sorted_by_yral_feed_score
            .replace(&post_1);
        canister_data
            .posts_index_sorted_by_yral_feed_score
            .replace(&post_2);

        let iter_posts = canister_data
            .posts_index_sorted_by_yral_feed_score
            .iter()
            .collect::<Vec<_>>();

        assert_eq!(iter_posts.len(), 2);
        assert_eq!(iter_posts[0], &post_2);
        assert_eq!(iter_posts[1], &post_1);

        let new_post_2 = PostScoreIndexItemV1 {
            post_id: 2,
            score: 10,
            publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            is_nsfw: true,
            status: PostStatus::BannedDueToUserReporting,
            created_at: Some(created_at_ealier),
        };

        update_post_yral_feed_impl(new_post_2.clone(), &mut canister_data);

        let iter_posts = canister_data
            .posts_index_sorted_by_yral_feed_score
            .iter()
            .collect::<Vec<_>>();

        assert_eq!(iter_posts.len(), 1);
        assert_eq!(iter_posts[0], &post_1);
    }
}
