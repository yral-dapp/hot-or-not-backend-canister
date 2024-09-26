use std::time::{Duration, SystemTime};

use ic_cdk_macros::update;
// use rand::Rng;
use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::{
        GlobalRoomId, RoomBetPossibleOutcomes, SlotId, DURATION_OF_EACH_SLOT_IN_SECONDS,
    },
    common::utils::permissions::is_caller_controller_or_global_admin,
    common::utils::system_time,
};

use crate::{data_model::CanisterData, CANISTER_DATA};

use super::tabulate_hot_or_not_outcome_for_post_slot::tabulate_hot_or_not_outcome_for_post_slot;

pub fn reenqueue_timers_for_pending_bet_outcomes() {
    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        let posts = get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        reenqueue_timers_for_these_posts(&mut canister_data, posts, &current_time);
    });
}

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

fn once_reenqueue_timers_for_these_posts(post_slot_ids: Vec<(u64, u8)>) {
    for (post_id, slot_id) in post_slot_ids {
        let slot_number = slot_id;

        ic_cdk_timers::set_timer(
            // random jitter
            Duration::from_secs(300),
            move || {
                tabulate_hot_or_not_outcome_for_post_slot(post_id, slot_number);
            },
        );
    }
}

fn once_get_posts_that_have_pending_outcomes(
    canister_data: &CanisterData,
    current_time: &SystemTime,
) -> Vec<(u64, u8)> {
    let room_details_map = &canister_data.room_details_map;
    canister_data
        .all_created_posts
        .iter()
        .rev()
        .flat_map(|(post_id, post)| {
            let created_outside_the_last_48_hours = current_time
                .duration_since(post.created_at)
                .unwrap_or(Duration::from_secs((48 * 60 + 5) * 60))
                .as_secs()
                > 48 * 60 * 60;
            let is_a_hot_or_not_post = post.hot_or_not_details.is_some();

            if created_outside_the_last_48_hours && is_a_hot_or_not_post {
                // scan through all the slots for hung timers
                let latest_room = 100_u64;

                let start_global_room_id = GlobalRoomId(post.id, 1, 1);
                let end_global_room_id = GlobalRoomId(post.id, 48_u8, latest_room);

                // for each post, there are many slot, each slot with many room details
                room_details_map
                    .range(start_global_room_id..end_global_room_id)
                    .filter_map(|(groomid, room_detail)| {
                        if matches!(room_detail.bet_outcome, RoomBetPossibleOutcomes::BetOngoing) {
                            Some((*post_id, groomid.1))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        })
        .collect()
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
    canister_data: &mut CanisterData,
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
                    tabulate_hot_or_not_outcome_for_post_slot(post_id, slot_number + 1);
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
            hot_or_not::{HotOrNotDetails, RoomDetailsV1},
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

    #[test]
    fn test_once_get_posts_that_have_pending_outcomes() {
        let mut canister_data = CanisterData::default();

        let current_time = SystemTime::now();
        let old_time = current_time - Duration::from_secs(49 * 60 * 60);

        // Create posts
        for i in 0..5 {
            let post = Post {
                id: i,
                created_at: if i % 2 != 0 {
                    old_time.checked_sub(Duration::from_secs(i * 60)).unwrap()
                } else {
                    current_time
                        .checked_sub(Duration::from_secs(3 * 60 * 60))
                        .unwrap()
                },
                // only those posts which have hot_or_not_details !=None and create_at > 48 hrs should filter through
                hot_or_not_details: if i % 2 != 0 {
                    Some(HotOrNotDetails::default())
                } else {
                    None
                },
                is_nsfw: false,
                description: "Singing and dancing".to_string(),
                hashtags: vec!["sing".to_string(), "dance".to_string()],
                video_uid: "video#0001".to_string(),
                status: PostStatus::ReadyToView,
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                home_feed_score: FeedScore::default(),
            };
            canister_data.all_created_posts.insert(i, post);
        }

        // Populate room_details_map
        let room_details_map = &mut canister_data.room_details_map;
        for post_id in 0..5 {
            for slot in 1..=48 {
                // just to ensure that some slot ids are empty.
                // i.e. in some slots, there wasn't a single bet placed
                if slot % 3 == 0 {
                    continue;
                }

                for room in 1..=3 {
                    let global_room_id = GlobalRoomId(post_id, slot, room);
                    let outcome = if room == 2 {
                        RoomBetPossibleOutcomes::BetOngoing
                    } else {
                        RoomBetPossibleOutcomes::HotWon
                    };
                    room_details_map.insert(
                        global_room_id,
                        RoomDetailsV1 {
                            bet_outcome: outcome,
                            ..Default::default()
                        },
                    );
                }
            }
        }

        let result = once_get_posts_that_have_pending_outcomes(&canister_data, &current_time);

        assert!(result.iter().all(|(post_id, _)| post_id % 2 != 0));
    }
}
