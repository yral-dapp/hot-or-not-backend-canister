use shared_utils::types::top_posts::post_score_index_item::v1::PostScoreIndexItem;

use crate::{data_model::CanisterDataV2, CANISTER_DATA_V2};

#[ic_cdk_macros::update]
#[candid::candid_method(update)]
fn receive_top_hot_or_not_feed_posts_from_publishing_canister(
    top_posts_from_publishing_canister: Vec<PostScoreIndexItem>,
) {
    // TODO: Add access control to allow only project canisters to send this message

    CANISTER_DATA_V2.with(|canister_data| {
        let mut canister_data = canister_data.borrow_mut();

        receive_top_hot_or_not_feed_posts_from_publishing_canister_impl(
            top_posts_from_publishing_canister,
            &mut canister_data,
        );
    });
}

fn receive_top_hot_or_not_feed_posts_from_publishing_canister_impl(
    top_posts_from_publishing_canister: Vec<PostScoreIndexItem>,
    canister_data: &mut CanisterDataV2,
) {
    let posts_index_sorted_by_hot_or_not_feed_score =
        &mut canister_data.posts_index_sorted_by_hot_or_not_feed_score;

    for post_score_index_item in top_posts_from_publishing_canister {
        posts_index_sorted_by_hot_or_not_feed_score.replace(&post_score_index_item);
    }

    if posts_index_sorted_by_hot_or_not_feed_score.iter().count() > 1500 {
        *posts_index_sorted_by_hot_or_not_feed_score = posts_index_sorted_by_hot_or_not_feed_score
            .into_iter()
            .take(1000)
            .cloned()
            .collect();
    }
}

#[cfg(test)]
mod test {
    use candid::Principal;

    use super::*;

    #[test]
    fn test_receive_top_hot_or_not_feed_posts_from_publishing_canister_impl() {
        let mut canister_data = CanisterDataV2::default();

        let top_posts_from_publishing_canister = vec![
            PostScoreIndexItem {
                post_id: 1,
                score: 1,
                publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            },
            PostScoreIndexItem {
                post_id: 3,
                score: 3,
                publisher_canister_id: Principal::anonymous(),
            },
            PostScoreIndexItem {
                post_id: 5,
                score: 5,
                publisher_canister_id: Principal::from_text("aaaaa-aa").unwrap(),
            },
        ];

        receive_top_hot_or_not_feed_posts_from_publishing_canister_impl(
            top_posts_from_publishing_canister,
            &mut canister_data,
        );

        let posts_index_sorted_by_hot_or_not_feed_score =
            &canister_data.posts_index_sorted_by_hot_or_not_feed_score;

        assert_eq!(
            posts_index_sorted_by_hot_or_not_feed_score.iter().count(),
            3
        );
    }
}
