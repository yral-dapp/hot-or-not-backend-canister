use crate::{data_model::CanisterData, CANISTER_DATA};
use shared_utils::{
    common::types::top_posts::post_score_index_item::PostScoreIndexItem,
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
) -> Result<Vec<PostScoreIndexItem>, TopPostsFetchError> {
    CANISTER_DATA.with(|canister_data| {
        let canister_data = canister_data.borrow();

        get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
            from_inclusive_index,
            limit,
            &canister_data,
        )
    })
}

fn get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
    from_inclusive_index: u64,
    limit: u64,
    canister_data: &CanisterData,
) -> Result<Vec<PostScoreIndexItem>, TopPostsFetchError> {
    let all_posts = &canister_data.posts_index_sorted_by_hot_or_not_feed_score;

    let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
        from_inclusive_index,
        limit,
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
        .skip(from_inclusive_index as usize)
        .take(limit as usize)
        .cloned()
        .collect::<Vec<PostScoreIndexItem>>())
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

        let post_scores = vec![
            PostScoreIndexItem {
                post_id: 1,
                score: 1,
                publisher_canister_id: Principal::anonymous(),
            },
            PostScoreIndexItem {
                post_id: 2,
                score: 2,
                publisher_canister_id: Principal::anonymous(),
            },
            PostScoreIndexItem {
                post_id: 3,
                score: 3,
                publisher_canister_id: Principal::anonymous(),
            },
            PostScoreIndexItem {
                post_id: 4,
                score: 4,
                publisher_canister_id: Principal::anonymous(),
            },
            PostScoreIndexItem {
                post_id: 5,
                score: 5,
                publisher_canister_id: Principal::anonymous(),
            },
        ];

        for post_score in post_scores {
            canister_data
                .posts_index_sorted_by_hot_or_not_feed_score
                .replace(&post_score);
        }

        // Test with cursor 0
        let result
            = super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
                    0,
                    3,
                    &canister_data);

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 3);
        assert_eq!(posts[0].post_id, 5);
        assert_eq!(posts[1].post_id, 4);
        assert_eq!(posts[2].post_id, 3);

        // Test with cursor 3
        let result
            = super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
                        3,
                        3,
                        &canister_data);

        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].post_id, 2);
        assert_eq!(posts[1].post_id, 1);

        // Test with cursor 5
        let result
            = super::get_top_posts_aggregated_from_canisters_on_this_network_for_hot_or_not_feed_cursor_impl(
                        5,
                        3,
                        &canister_data);

        assert_eq!(result, Err(TopPostsFetchError::ReachedEndOfItemsList));
    }
}
