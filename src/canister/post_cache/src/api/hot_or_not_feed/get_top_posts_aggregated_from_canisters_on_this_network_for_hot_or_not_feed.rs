use crate::{data_model::CanisterData, CANISTER_DATA};
use shared_utils::{
    common::types::top_posts::post_score_index_item::{
        PostScoreIndexItem, PostScoreIndexItemV1, PostStatus,
    },
    pagination::{self, PaginationError},
    types::canister_specific::post_cache::error_types::TopPostsFetchError,
};

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed(
    from_inclusive_index: u64,
    to_exclusive_index: u64,
) -> Result<Vec<PostScoreIndexItem>, TopPostsFetchError> {
    CANISTER_DATA.with(|canister_data| {
        let canister_data = canister_data.borrow();

        get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_impl(
            from_inclusive_index,
            to_exclusive_index,
            &canister_data,
        )
    })
}

fn get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_impl(
    from_inclusive_index: u64,
    to_exclusive_index: u64,
    canister_data: &CanisterData,
) -> Result<Vec<PostScoreIndexItem>, TopPostsFetchError> {
    let all_posts = &canister_data.posts_index_sorted_by_hot_or_not_feed_score;

    let (from_inclusive_index, to_exclusive_index) = pagination::get_pagination_bounds(
        from_inclusive_index,
        to_exclusive_index,
        all_posts.iter().count() as u64,
    )
    .map_err(|e| match e {
        PaginationError::InvalidBoundsPassed => TopPostsFetchError::InvalidBoundsPassed,
        PaginationError::ReachedEndOfItemsList => TopPostsFetchError::ReachedEndOfItemsList,
        PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest => {
            TopPostsFetchError::ExceededMaxNumberOfItemsAllowedInOneRequest
        }
    })?;

    Ok(all_posts
        .iter()
        .take(to_exclusive_index as usize)
        .skip(from_inclusive_index as usize)
        .cloned()
        .collect())
}

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor(
    from_inclusive_index: u64,
    limit: u64,
    is_nsfw: Option<bool>,
    status: Option<PostStatus>,
) -> Result<Vec<PostScoreIndexItemV1>, TopPostsFetchError> {
    CANISTER_DATA.with(|canister_data| {
        let canister_data = canister_data.borrow();

        get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
            from_inclusive_index,
            limit,
            &canister_data,
            is_nsfw,
            status,
        )
    })
}

fn get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
    from_inclusive_index: u64,
    limit: u64,
    canister_data: &CanisterData,
    is_nsfw: Option<bool>,
    status: Option<PostStatus>,
) -> Result<Vec<PostScoreIndexItemV1>, TopPostsFetchError> {
    let all_posts = &canister_data.posts_index_sorted_by_hot_or_not_feed_score_v1;
    let filter_fn = |post_item: &PostScoreIndexItemV1| {
        if let Some(is_nsfw) = is_nsfw {
            if post_item.is_nsfw != is_nsfw {
                return false;
            }
        }

        if let Some(status) = status.clone() {
            if post_item.status != status {
                return false;
            }
        }

        true
    };

    let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
        from_inclusive_index,
        limit,
        all_posts
            .iter()
            .filter(|&post_item| filter_fn(post_item))
            .count() as u64,
    )
    .map_err(|e| match e {
        PaginationError::InvalidBoundsPassed => TopPostsFetchError::InvalidBoundsPassed,
        PaginationError::ReachedEndOfItemsList => TopPostsFetchError::ReachedEndOfItemsList,
        PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest => {
            TopPostsFetchError::ExceededMaxNumberOfItemsAllowedInOneRequest
        }
    })?;

    Ok(all_posts
        .iter()
        .filter(|&post_item| filter_fn(post_item))
        .skip(from_inclusive_index as usize)
        .take(limit as usize)
        .cloned()
        .collect::<Vec<PostScoreIndexItemV1>>())
}

#[cfg(test)]
mod test {
    use candid::Principal;

    use super::*;

    #[test]
    fn test_get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_impl() {
        let mut canister_data = CanisterData::default();

        let result =
            super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_impl(
                0,
                10,
                &canister_data,
            );

        assert!(result.is_err());
        assert_eq!(
            result,
            Err(super::TopPostsFetchError::ReachedEndOfItemsList)
        );

        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&PostScoreIndexItem {
                post_id: 1,
                score: 1,
                publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            });

        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&PostScoreIndexItem {
                post_id: 1,
                score: 2,
                publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            });

        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&PostScoreIndexItem {
                post_id: 2,
                score: 5,
                publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            });

        assert!(super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_impl(
            0,
            10,
            &canister_data
        ).is_ok());
        assert!(
            super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_impl(
                0,
                10,
                &canister_data
            )
            .unwrap()
            .len() == 2
        );
    }

    #[test]
    fn test_get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_impl_with_indexes(
    ) {
        let mut canister_data = CanisterData::default();

        let post_score_index_item_1 = PostScoreIndexItem {
            post_id: 1,
            score: 1,
            publisher_canister_id: Principal::anonymous(),
        };
        let post_score_index_item_2 = PostScoreIndexItem {
            post_id: 2,
            score: 2,
            publisher_canister_id: Principal::anonymous(),
        };
        let post_score_index_item_3 = PostScoreIndexItem {
            post_id: 3,
            score: 3,
            publisher_canister_id: Principal::anonymous(),
        };
        let post_score_index_item_4 = PostScoreIndexItem {
            post_id: 4,
            score: 4,
            publisher_canister_id: Principal::anonymous(),
        };
        let post_score_index_item_5 = PostScoreIndexItem {
            post_id: 5,
            score: 5,
            publisher_canister_id: Principal::anonymous(),
        };

        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&post_score_index_item_1);
        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&post_score_index_item_2);

        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&post_score_index_item_3);

        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&post_score_index_item_4);

        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&post_score_index_item_5);

        let result =
            super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_impl(
                2,
                3,
                &canister_data,
            );
        assert!(result.is_ok());

        let posts = result.unwrap();
        assert_eq!(posts.len(), 1);

        let third_post = posts.get(0).unwrap();
        assert_eq!(third_post.post_id, 3);
        assert_eq!(third_post.score, 3);
    }

    #[test]
    fn test_get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
    ) {
        let mut canister_data = CanisterData::default();
        let created_at_now = std::time::SystemTime::now();
        let created_at_earlier = created_at_now - std::time::Duration::from_secs(48 * 60 * 60 + 1);

        let post_scores = vec![
            PostScoreIndexItemV1 {
                post_id: 1,
                score: 1,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_now),
            },
            PostScoreIndexItemV1 {
                post_id: 2,
                score: 2,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_now),
            },
            PostScoreIndexItemV1 {
                post_id: 3,
                score: 3,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_earlier),
            },
            PostScoreIndexItemV1 {
                post_id: 4,
                score: 4,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_earlier),
            },
            PostScoreIndexItemV1 {
                post_id: 5,
                score: 5,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_earlier),
            },
            PostScoreIndexItemV1 {
                post_id: 1,
                score: 6,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: true,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_earlier),
            },
        ];

        for post_score in post_scores {
            canister_data
                .posts_index_sorted_by_hot_or_not_feed_score_v1
                .replace(&post_score);
        }

        // Test with cursor 0
        let result
            = super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
                    0,
                    10,
                    &canister_data, None, None);

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 5);
        assert_eq!(posts[0].post_id, 2);
        assert_eq!(posts[1].post_id, 1);
        assert_eq!(posts[2].post_id, 5);
        assert_eq!(posts[3].post_id, 4);
        assert_eq!(posts[4].post_id, 3);

        // Test with cursor 3
        let result
            = super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
                        3,
                        3,
                        &canister_data, None, None);

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].post_id, 4);
        assert_eq!(posts[1].post_id, 3);

        // Test with cursor 5
        let result
            = super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
                        5,
                        3,
                        &canister_data, None, None);

        assert_eq!(result, Err(TopPostsFetchError::ReachedEndOfItemsList));
    }

    #[test]
    fn test_get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl_with_nsfw_filter(
    ) {
        let mut canister_data = CanisterData::default();
        let created_at_now = std::time::SystemTime::now();
        let created_at_earlier = created_at_now - std::time::Duration::from_secs(48 * 60 * 60 + 1);

        let post_scores = vec![
            PostScoreIndexItemV1 {
                post_id: 1,
                score: 1,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_now),
            },
            PostScoreIndexItemV1 {
                post_id: 2,
                score: 2,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_now),
            },
            PostScoreIndexItemV1 {
                post_id: 3,
                score: 3,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: true,
                status: PostStatus::Deleted,
                created_at: Some(created_at_earlier),
            },
            PostScoreIndexItemV1 {
                post_id: 4,
                score: 4,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: true,
                status: PostStatus::Uploaded,
                created_at: Some(created_at_earlier),
            },
            PostScoreIndexItemV1 {
                post_id: 5,
                score: 5,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: false,
                status: PostStatus::ReadyToView,
                created_at: Some(created_at_earlier),
            },
            PostScoreIndexItemV1 {
                post_id: 1,
                score: 6,
                publisher_canister_id: Principal::anonymous(),
                is_nsfw: true,
                status: PostStatus::Deleted,
                created_at: Some(created_at_earlier),
            },
        ];

        for post_score in post_scores {
            canister_data
                .posts_index_sorted_by_hot_or_not_feed_score_v1
                .replace(&post_score);
        }

        // Test with NSFW filter
        let result
            = super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
                    0,
                    3,
                    &canister_data, Some(false), None);

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].post_id, 2);
        assert_eq!(posts[1].post_id, 5);

        // Test with status filter
        let result
            = super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
                    0,
                    3,
                    &canister_data, None, Some(PostStatus::Uploaded));

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].post_id, 4);

        // Test with both filters

        let result
            = super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
                    0,
                    3,
                    &canister_data, Some(true), Some(PostStatus::Deleted));

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].post_id, 1);
        assert_eq!(posts[1].post_id, 3);
    }
}
