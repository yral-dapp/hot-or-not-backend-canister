use std::time::SystemTime;

use candid::Principal;
use ic_cdk::api::call;
use shared_utils::{
    common::{
        types::{
            known_principal::KnownPrincipalType,
            top_posts::post_score_index_item::PostScoreIndexItem,
        },
        utils::system_time,
    },
    constant::{
        HOME_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION,
        HOT_OR_NOT_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION,
    },
};

use crate::{data_model::CanisterData, CANISTER_DATA};

pub fn update_scores_and_share_with_post_cache_if_difference_beyond_threshold(post_id: u64) {
    let current_time = system_time::get_current_system_time_from_ic();
    let canisters_own_principal_id = ic_cdk::id();

    let (home_feed_index_score_item, hot_or_not_index_score_item): (
        Option<PostScoreIndexItem>,
        Option<PostScoreIndexItem>,
    ) = CANISTER_DATA.with(|canister_data_ref_cell| {
        update_home_feed_and_hot_or_not_feed_score_and_get_post_index_item_to_send(
            &mut canister_data_ref_cell.borrow_mut(),
            post_id,
            current_time,
            canisters_own_principal_id,
        )
    });

    if home_feed_index_score_item.is_none() && hot_or_not_index_score_item.is_none() {
        return;
    }

    let post_cache_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdPostCache)
            .cloned()
            .unwrap()
    });

    if home_feed_index_score_item.is_some() {
        let _ = call::notify(
            post_cache_canister_principal_id,
            "receive_top_home_feed_posts_from_publishing_canister",
            (vec![home_feed_index_score_item.unwrap()],),
        );
    }

    if hot_or_not_index_score_item.is_some() {
        let _ = call::notify(
            post_cache_canister_principal_id,
            "receive_top_hot_or_not_feed_posts_from_publishing_canister",
            (vec![hot_or_not_index_score_item.unwrap()],),
        );
    }
}

fn update_home_feed_and_hot_or_not_feed_score_and_get_post_index_item_to_send(
    canister_data: &mut CanisterData,
    post_id: u64,
    current_time: SystemTime,
    canisters_own_principal_id: Principal,
) -> (Option<PostScoreIndexItem>, Option<PostScoreIndexItem>) {
    let all_posts = &mut canister_data.all_created_posts;
    if !all_posts.contains_key(&post_id) {
        return (None, None);
    }

    let mut home_feed_index_score_item: Option<PostScoreIndexItem> = None;
    let mut hot_or_not_index_score_item: Option<PostScoreIndexItem> = None;

    let mut post_to_synchronise = all_posts.get(&post_id).unwrap().clone();

    post_to_synchronise.recalculate_home_feed_score(&current_time);

    let last_updated_home_feed_score = post_to_synchronise.home_feed_score.last_synchronized_score;
    let current_home_feed_score = post_to_synchronise.home_feed_score.current_score;

    let home_feed_score_difference = current_home_feed_score.abs_diff(last_updated_home_feed_score);

    if home_feed_score_difference > HOME_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION {
        home_feed_index_score_item = Some(PostScoreIndexItem {
            post_id: post_to_synchronise.id,
            score: current_home_feed_score,
            publisher_canister_id: canisters_own_principal_id,
        });
        post_to_synchronise.home_feed_score.last_synchronized_score = current_home_feed_score;
        post_to_synchronise.home_feed_score.last_synchronized_at = current_time;
    }

    if post_to_synchronise.hot_or_not_details.is_some() {
        post_to_synchronise.recalculate_hot_or_not_feed_score(&current_time);
        let last_updated_hot_or_not_feed_score = post_to_synchronise
            .hot_or_not_details
            .as_ref()
            .unwrap()
            .hot_or_not_feed_score
            .last_synchronized_score;
        let current_hot_or_not_feed_score = post_to_synchronise
            .hot_or_not_details
            .as_ref()
            .unwrap()
            .hot_or_not_feed_score
            .current_score;

        let hot_or_not_feed_score_difference =
            current_hot_or_not_feed_score.abs_diff(last_updated_hot_or_not_feed_score);

        if hot_or_not_feed_score_difference > HOT_OR_NOT_FEED_DIFFERENCE_TO_INITIATE_SYNCHRONISATION
        {
            hot_or_not_index_score_item = Some(PostScoreIndexItem {
                post_id: post_to_synchronise.id,
                score: current_hot_or_not_feed_score,
                publisher_canister_id: canisters_own_principal_id,
            });
            post_to_synchronise
                .hot_or_not_details
                .as_mut()
                .unwrap()
                .hot_or_not_feed_score
                .last_synchronized_score = current_hot_or_not_feed_score;
            post_to_synchronise
                .hot_or_not_details
                .as_mut()
                .unwrap()
                .hot_or_not_feed_score
                .last_synchronized_at = current_time;
        }
    }

    all_posts.insert(post_id, post_to_synchronise);

    (home_feed_index_score_item, hot_or_not_index_score_item)
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use shared_utils::canister_specific::individual_user_template::types::post::{
        Post, PostDetailsFromFrontend,
    };
    use test_utils::setup::test_constants::get_mock_user_alice_canister_id;

    use super::*;

    #[test]
    fn test_update_home_feed_and_hot_or_not_feed_score_and_get_post_index_item_to_send() {
        let mut canister_data = CanisterData::default();
        let post_creation_time = SystemTime::now();

        let response = update_home_feed_and_hot_or_not_feed_score_and_get_post_index_item_to_send(
            &mut canister_data,
            0,
            post_creation_time,
            get_mock_user_alice_canister_id(),
        );
        assert_eq!(response, (None, None));

        canister_data.all_created_posts.insert(
            0,
            Post::new(
                0,
                PostDetailsFromFrontend {
                    description: "This is a new post".to_string(),
                    hashtags: vec!["#fun".to_string(), "#post".to_string()],
                    video_uid: "abcd1234".to_string(),
                    creator_consent_for_inclusion_in_hot_or_not: true,
                },
                &post_creation_time,
            ),
        );

        let response = update_home_feed_and_hot_or_not_feed_score_and_get_post_index_item_to_send(
            &mut canister_data,
            0,
            post_creation_time,
            get_mock_user_alice_canister_id(),
        );
        assert!(response.0.is_some());
        assert!(response.1.is_some());

        let response = update_home_feed_and_hot_or_not_feed_score_and_get_post_index_item_to_send(
            &mut canister_data,
            0,
            post_creation_time
                .checked_add(Duration::from_secs(120))
                .unwrap(),
            get_mock_user_alice_canister_id(),
        );
        assert_eq!(response, (None, None));
    }
}
