use std::time::{Duration, SystemTime};

use shared_utils::common::utils::system_time;

use crate::{data_model::CanisterData, CANISTER_DATA};

use super::tabulate_hot_or_not_outcome_for_post_slot::tabulate_hot_or_not_outcome_for_post_slot;

pub fn reenqueue_timers_for_pending_bet_outcomes() {
    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.borrow();

        let posts = get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        reenqueue_timers_for_these_posts(&canister_data, posts, &current_time);
    });
}

fn get_posts_that_have_pending_outcomes(
    canister_data: &CanisterData,
    current_time: &SystemTime,
) -> Vec<u64> {
    canister_data
        .all_created_posts
        .iter()
        .rev()
        .take_while(|(_post_id, post)| {
            let created_in_the_last_48_hours = current_time
                .duration_since(post.created_at)
                .unwrap_or(Duration::from_secs((48 * 60 + 5) * 60))
                .as_secs()
                < 48 * 60 * 60;
            let is_a_hot_or_not_post = post.hot_or_not_details.is_some();

            created_in_the_last_48_hours && is_a_hot_or_not_post
        })
        .map(|(post_id, _post)| *post_id)
        .collect()
}

fn reenqueue_timers_for_these_posts(
    canister_data: &CanisterData,
    post_ids: Vec<u64>,
    current_time: &SystemTime,
) {
    for post_id in post_ids {
        let post = canister_data.all_created_posts.get(&post_id).unwrap();

        let slot_to_enqueue_onwards = (current_time
            .duration_since(post.created_at)
            .unwrap()
            .as_secs()
            / (60 * 60)) as u8;

        // * schedule hot_or_not outcome tabulation for the 48 hours after the post is created
        (slot_to_enqueue_onwards..=48).for_each(|slot_number| {
            ic_cdk_timers::set_timer(
                post.created_at
                    .checked_add(Duration::from_secs((slot_number as u64) * 60 * 60))
                    .unwrap()
                    .duration_since(*current_time)
                    .unwrap_or_default(),
                move || {
                    CANISTER_DATA.with(|canister_data_ref_cell| {
                        tabulate_hot_or_not_outcome_for_post_slot(
                            &mut canister_data_ref_cell.borrow_mut(),
                            post_id,
                            slot_number + 1,
                        );
                    });
                },
            );
        })
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use shared_utils::{
        canister_specific::individual_user_template::types::{
            hot_or_not::HotOrNotDetails,
            post::{FeedScore, Post, PostViewStatistics},
        },
        common::types::top_posts::post_score_index_item::PostStatus,
    };

    use super::*;

    #[test]
    fn test_get_posts_that_have_pending_outcomes_case_1() {
        let mut canister_data = CanisterData::default();
        let post_0_creation_time = SystemTime::now();

        let post_0 = Post {
            id: 0,
            is_nsfw: false,
            description: "Singing and dancing".to_string(),
            hashtags: vec!["sing".to_string(), "dance".to_string()],
            video_uid: "video#0001".to_string(),
            status: PostStatus::ReadyToView,
            created_at: post_0_creation_time,
            likes: HashSet::new(),
            share_count: 0,
            view_stats: PostViewStatistics::default(),
            home_feed_score: FeedScore::default(),
            creator_consent_for_inclusion_in_hot_or_not: true,
            hot_or_not_details: Some(HotOrNotDetails::default()),
        };

        canister_data
            .all_created_posts
            .insert(canister_data.all_created_posts.len() as u64, post_0);
        let current_time = post_0_creation_time
            .checked_add(Duration::from_secs((48 * 60) * 60))
            .unwrap();

        let posts_that_have_pending_outcomes =
            get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        assert_eq!(posts_that_have_pending_outcomes.len(), 0);

        let current_time = post_0_creation_time
            .checked_add(Duration::from_secs(((48 * 60) - 1) * 60))
            .unwrap();

        let posts_that_have_pending_outcomes =
            get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        assert_eq!(posts_that_have_pending_outcomes.len(), 1);
        assert_eq!(posts_that_have_pending_outcomes[0], 0);

        let current_time = post_0_creation_time
            .checked_add(Duration::from_secs(((48 * 60) + 1) * 60))
            .unwrap();

        let posts_that_have_pending_outcomes =
            get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        assert_eq!(posts_that_have_pending_outcomes.len(), 0);

        let post_1 = Post {
            id: 1,
            is_nsfw: false,
            description: "Singing and dancing".to_string(),
            hashtags: vec!["sing".to_string(), "dance".to_string()],
            video_uid: "video#0001".to_string(),
            status: PostStatus::ReadyToView,
            created_at: post_0_creation_time
                .checked_add(Duration::from_secs(60 * 60))
                .unwrap(),
            likes: HashSet::new(),
            share_count: 0,
            view_stats: PostViewStatistics::default(),
            home_feed_score: FeedScore::default(),
            creator_consent_for_inclusion_in_hot_or_not: true,
            hot_or_not_details: Some(HotOrNotDetails::default()),
        };

        canister_data
            .all_created_posts
            .insert(canister_data.all_created_posts.len() as u64, post_1);

        let current_time = post_0_creation_time
            .checked_add(Duration::from_secs(((48 * 60) + 1) * 60))
            .unwrap();

        let posts_that_have_pending_outcomes =
            get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        assert_eq!(posts_that_have_pending_outcomes.len(), 1);
        assert_eq!(posts_that_have_pending_outcomes[0], 1);

        let current_time = post_0_creation_time
            .checked_add(Duration::from_secs((48 * 60) * 60))
            .unwrap();

        let posts_that_have_pending_outcomes =
            get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        assert_eq!(posts_that_have_pending_outcomes.len(), 1);
        assert_eq!(posts_that_have_pending_outcomes[0], 1);

        let current_time = post_0_creation_time
            .checked_add(Duration::from_secs(((48 * 60) - 1) * 60))
            .unwrap();

        let posts_that_have_pending_outcomes =
            get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        assert_eq!(posts_that_have_pending_outcomes.len(), 2);
        assert_eq!(posts_that_have_pending_outcomes[0], 1);
        assert_eq!(posts_that_have_pending_outcomes[1], 0);

        let post_2 = Post {
            id: 2,
            is_nsfw: false,
            description: "Singing and dancing".to_string(),
            hashtags: vec!["sing".to_string(), "dance".to_string()],
            video_uid: "video#0001".to_string(),
            status: PostStatus::ReadyToView,
            created_at: post_0_creation_time
                .checked_add(Duration::from_secs(((2 * 60) + 5) * 60))
                .unwrap(),
            likes: HashSet::new(),
            share_count: 0,
            view_stats: PostViewStatistics::default(),
            home_feed_score: FeedScore::default(),
            creator_consent_for_inclusion_in_hot_or_not: true,
            hot_or_not_details: Some(HotOrNotDetails::default()),
        };

        canister_data
            .all_created_posts
            .insert(canister_data.all_created_posts.len() as u64, post_2);

        let current_time = post_0_creation_time
            .checked_add(Duration::from_secs(((48 * 60) - 1) * 60))
            .unwrap();

        let posts_that_have_pending_outcomes =
            get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        assert_eq!(posts_that_have_pending_outcomes.len(), 3);
        assert_eq!(posts_that_have_pending_outcomes[0], 2);
        assert_eq!(posts_that_have_pending_outcomes[1], 1);
        assert_eq!(posts_that_have_pending_outcomes[2], 0);
    }
}
