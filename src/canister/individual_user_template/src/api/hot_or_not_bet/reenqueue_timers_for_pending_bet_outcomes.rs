use std::{
    cmp::Ordering,
    collections::HashSet,
    time::{Duration, SystemTime},
};

use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::{
        RoomBetPossibleOutcomes, DURATION_OF_EACH_SLOT_IN_SECONDS,
    },
    common::utils::permissions::is_caller_controller_or_global_admin,
    common::utils::system_time,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

use super::tabulate_hot_or_not_outcome_for_post_slot::{
    tabulate_hot_or_not_outcome_for_post_slot, tabulate_hot_or_not_outcome_for_post_slot_v1,
};

pub fn reenqueue_timers_for_pending_bet_outcomes() {
    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow(|canister_data| {
        let posts = canister_data.get_posts_that_have_pending_outcomes();
        reenqueue_timers_for_these_posts(canister_data, posts, &current_time);
    })
}

fn reenqueue_timers_for_these_posts(
    canister_data: &CanisterData,
    post_ids: Vec<u64>,
    current_time: &SystemTime,
) {
    for post_id in post_ids {
        let post = canister_data.get_post(&post_id).unwrap();

        post.slots_left_to_be_computed
            .iter()
            .for_each(|slot_number| {
                let slot_id = *slot_number;
                ic_cdk_timers::set_timer(
                    post.created_at
                        .checked_add(Duration::from_secs(
                            (slot_id as u64) * DURATION_OF_EACH_SLOT_IN_SECONDS,
                        ))
                        .unwrap()
                        .duration_since(*current_time)
                        .unwrap_or_default(),
                    move || {
                        ic_cdk::spawn(tabulate_hot_or_not_outcome_for_post_slot(post_id, slot_id));
                        ic_cdk::spawn(tabulate_hot_or_not_outcome_for_post_slot_v1(
                            post_id, slot_id,
                        ));
                    },
                );
            })
    }
}

#[deprecated]
#[update(guard = "is_caller_controller_or_global_admin")]
async fn once_reenqueue_timers_for_pending_bet_outcomes() -> Result<Vec<(u64, u8)>, String> {
    let current_time = system_time::get_current_system_time_from_ic();

    let post_w_slot = CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.borrow_mut();

        let posts_with_slots =
            once_get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        // once_reenqueue_timers_for_these_posts(&mut canister_data, posts_with_slots, &current_time);
        once_reenqueue_timers_for_these_posts(posts_with_slots.clone());
        posts_with_slots
    });

    Ok(post_w_slot)
}

#[deprecated]
fn once_reenqueue_timers_for_these_posts(post_slot_ids: Vec<(u64, u8)>) {
    for (post_id, slot_id) in post_slot_ids {
        let slot_number = slot_id;

        ic_cdk_timers::set_timer(
            // random jitter
            Duration::from_secs(300),
            move || {
                ic_cdk::spawn(tabulate_hot_or_not_outcome_for_post_slot(
                    post_id,
                    slot_number,
                ));
            },
        );
    }
}

#[deprecated]
fn once_get_posts_that_have_pending_outcomes(
    canister_data: &CanisterData,
    current_time: &SystemTime,
) -> Vec<(u64, u8)> {
    let room_details_map = &canister_data.room_details_map;
    let post_and_slot_left_for_computation: HashSet<(u64, u8)> = room_details_map
        .iter()
        .filter(|(global_room_id, room_details)| {
            let bet_ongoing = room_details.bet_outcome == RoomBetPossibleOutcomes::BetOngoing;

            if !bet_ongoing {
                return false;
            }

            let slot_id = global_room_id.1;

            let post_created_time = canister_data
                .get_post(&global_room_id.0)
                .map(|post| post.created_at);

            if let Some(post_created_time) = post_created_time {
                let slot_computation_time =
                    post_created_time.checked_add(Duration::from_secs(slot_id as u64 * 65 * 60)); // 5 minutes more for buffer

                let has_slot_passed = slot_computation_time.map(|slot_trigger_time| {
                    match slot_trigger_time.cmp(current_time) {
                        Ordering::Less => true,
                        _ => false,
                    }
                });

                has_slot_passed.unwrap_or(false)
            } else {
                return false;
            }
        })
        .map(|(global_room_id, _)| (global_room_id.0, global_room_id.1))
        .collect();

    post_and_slot_left_for_computation.into_iter().collect()
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
            hot_or_not_details: Some(HotOrNotDetails::default()),
            slots_left_to_be_computed: HashSet::new(),
        };

        canister_data.add_post(post_0);

        let posts_that_have_pending_outcomes = canister_data.get_posts_that_have_pending_outcomes();

        assert_eq!(posts_that_have_pending_outcomes.len(), 0);

        let post_1 = Post {
            id: 1,
            is_nsfw: false,
            description: "Singing and dancing".to_string(),
            hashtags: vec!["sing".to_string(), "dance".to_string()],
            video_uid: "video#0001".to_string(),
            status: PostStatus::ReadyToView,
            created_at: post_0_creation_time
                .checked_add(Duration::from_secs(DURATION_OF_EACH_SLOT_IN_SECONDS))
                .unwrap(),
            likes: HashSet::new(),
            share_count: 0,
            view_stats: PostViewStatistics::default(),
            home_feed_score: FeedScore::default(),
            hot_or_not_details: Some(HotOrNotDetails::default()),
            slots_left_to_be_computed: (1..=48).collect(),
        };

        canister_data.add_post(post_1);

        let posts_that_have_pending_outcomes = canister_data.get_posts_that_have_pending_outcomes();

        assert_eq!(posts_that_have_pending_outcomes.len(), 1);
        assert_eq!(posts_that_have_pending_outcomes[0], 1);

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
            hot_or_not_details: Some(HotOrNotDetails::default()),
            slots_left_to_be_computed: (10..=48).collect(),
        };

        canister_data.add_post(post_2);

        let posts_that_have_pending_outcomes = canister_data.get_posts_that_have_pending_outcomes();

        assert_eq!(posts_that_have_pending_outcomes.len(), 2);
        assert_eq!(posts_that_have_pending_outcomes[0], 1);
        assert_eq!(posts_that_have_pending_outcomes[1], 2);
    }
}
