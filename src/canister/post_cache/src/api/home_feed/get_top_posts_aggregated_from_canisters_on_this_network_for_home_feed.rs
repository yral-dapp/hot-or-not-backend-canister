use crate::{data_model::CanisterDataV2, CANISTER_DATA_V2};
use shared_utils::{
    pagination::{self, PaginationError},
    types::{
        canister_specific::post_cache::error_types::TopPostsFetchError,
        top_posts::post_score_index_item::v1::PostScoreIndexItem,
    },
};

#[ic_cdk_macros::query]
#[candid::candid_method(query)]
fn get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed(
    from_inclusive_index: u64,
    to_exclusive_index: u64,
) -> Result<Vec<PostScoreIndexItem>, TopPostsFetchError> {
    CANISTER_DATA_V2.with(|canister_data| {
        let canister_data = canister_data.borrow();

        get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed_impl(
            from_inclusive_index,
            to_exclusive_index,
            &canister_data,
        )
    })
}

fn get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed_impl(
    from_inclusive_index: u64,
    to_exclusive_index: u64,
    canister_data: &CanisterDataV2,
) -> Result<Vec<PostScoreIndexItem>, TopPostsFetchError> {
    let all_posts = &canister_data.posts_index_sorted_by_home_feed_score;

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
        .skip(from_inclusive_index as usize)
        .take(to_exclusive_index as usize)
        .cloned()
        .collect())
}

#[cfg(test)]
mod test {
    use candid::Principal;

    use super::*;

    #[test]
    fn test_get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed() {
        let mut canister_data = CanisterDataV2::default();

        let result =
            super::get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed_impl(
                0,
                10,
                &canister_data,
            );
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            super::TopPostsFetchError::ReachedEndOfItemsList
        );

        let post_score_index_item_1 = PostScoreIndexItem {
            post_id: 1,
            score: 1,
            publisher_canister_id: Principal::anonymous(),
        };
        let post_score_index_item_2 = PostScoreIndexItem {
            post_id: 1,
            score: 2,
            publisher_canister_id: Principal::anonymous(),
        };
        let post_score_index_item_3 = PostScoreIndexItem {
            post_id: 2,
            score: 3,
            publisher_canister_id: Principal::anonymous(),
        };
        canister_data
            .posts_index_sorted_by_home_feed_score
            .replace(&post_score_index_item_1);
        canister_data
            .posts_index_sorted_by_home_feed_score
            .replace(&post_score_index_item_2);
        canister_data
            .posts_index_sorted_by_home_feed_score
            .replace(&post_score_index_item_3);

        let result =
            super::get_top_posts_aggregated_from_canisters_on_this_network_for_home_feed_impl(
                0,
                10,
                &canister_data,
            );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2,);
    }
}
