use shared_utils::common::types::{
    known_principal::KnownPrincipalType, top_posts::post_score_index::PostScoreIndex,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn remove_all_feed_entries() {
    let api_caller = ic_cdk::caller();

    let super_admin_user = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .unwrap()
            .clone()
    });

    if api_caller != super_admin_user {
        return;
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        remove_all_feed_entries_impl(&mut canister_data);
    });
}

fn remove_all_feed_entries_impl(canister_data: &mut CanisterData) {
    canister_data.posts_index_sorted_by_home_feed_score = PostScoreIndex::default();
    canister_data.posts_index_sorted_by_hot_or_not_feed_score = PostScoreIndex::default();
}

#[cfg(test)]
mod test {
    use shared_utils::common::types::top_posts::post_score_index_item::PostScoreIndexItem;
    use test_utils::setup::test_constants::get_mock_user_alice_canister_id;

    use super::*;

    #[test]
    fn test_remove_all_feed_entries_impl() {
        let mut canister_data = CanisterData::default();

        remove_all_feed_entries_impl(&mut canister_data);

        assert_eq!(
            canister_data
                .posts_index_sorted_by_home_feed_score
                .iter()
                .count(),
            0
        );
        assert_eq!(
            canister_data
                .posts_index_sorted_by_hot_or_not_feed_score
                .iter()
                .count(),
            0
        );

        canister_data
            .posts_index_sorted_by_home_feed_score
            .replace(&PostScoreIndexItem {
                post_id: 0,
                publisher_canister_id: get_mock_user_alice_canister_id(),
                score: 100,
            });
        canister_data
            .posts_index_sorted_by_home_feed_score
            .replace(&PostScoreIndexItem {
                post_id: 1,
                publisher_canister_id: get_mock_user_alice_canister_id(),
                score: 200,
            });

        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&PostScoreIndexItem {
                post_id: 0,
                publisher_canister_id: get_mock_user_alice_canister_id(),
                score: 100,
            });
        canister_data
            .posts_index_sorted_by_hot_or_not_feed_score
            .replace(&PostScoreIndexItem {
                post_id: 1,
                publisher_canister_id: get_mock_user_alice_canister_id(),
                score: 200,
            });

        assert_eq!(
            canister_data
                .posts_index_sorted_by_home_feed_score
                .iter()
                .count(),
            2
        );
        assert_eq!(
            canister_data
                .posts_index_sorted_by_hot_or_not_feed_score
                .iter()
                .count(),
            2
        );

        remove_all_feed_entries_impl(&mut canister_data);

        assert_eq!(
            canister_data
                .posts_index_sorted_by_home_feed_score
                .iter()
                .count(),
            0
        );
        assert_eq!(
            canister_data
                .posts_index_sorted_by_hot_or_not_feed_score
                .iter()
                .count(),
            0
        );
    }
}
