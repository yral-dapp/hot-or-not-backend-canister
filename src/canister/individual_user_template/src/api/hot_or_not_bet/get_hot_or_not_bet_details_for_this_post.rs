use ic_cdk_macros::query;
use std::time::SystemTime;

use candid::Principal;
use shared_utils::{
    canister_specific::individual_user_template::types::{cents, hot_or_not::BettingStatus},
    common::utils::system_time::{self},
};

use crate::{
    data_model::{
        cents_hot_or_not_game::{self, CentsHotOrNotGame},
        pump_n_dump::TokenBetGame,
        CanisterData,
    },
    CANISTER_DATA, PUMP_N_DUMP,
};

#[deprecated]
#[query]
fn get_hot_or_not_bet_details_for_this_post(post_id: u64) -> BettingStatus {
    let request_maker = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        get_hot_or_not_bet_details_for_this_post_impl(
            &canister_data_ref_cell.borrow(),
            &system_time::get_current_system_time_from_ic(),
            &request_maker,
            post_id,
        )
    })
}

#[query]
fn get_hot_or_not_bet_details_for_this_post_v1(post_id: u64) -> BettingStatus {
    let request_maker = ic_cdk::caller();
    PUMP_N_DUMP.with_borrow_mut(|token_bet_game| {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            let cents_hot_or_not_game = CentsHotOrNotGame {
                canister_data,
                token_bet_game,
            };
            get_hot_or_not_bet_details_for_this_post_impl_v1(
                cents_hot_or_not_game,
                &system_time::get_current_system_time_from_ic(),
                &request_maker,
                post_id,
            )
        })
    })
}

#[deprecated]
fn get_hot_or_not_bet_details_for_this_post_impl(
    canister_data: &CanisterData,
    current_time: &SystemTime,
    request_maker: &Principal,
    post_id: u64,
) -> BettingStatus {
    let post = canister_data.get_post(&post_id).unwrap();

    post.get_hot_or_not_betting_status_for_this_post_v1(
        current_time,
        request_maker,
        &canister_data.room_details_map,
        &canister_data.post_principal_map,
        &canister_data.slot_details_map,
    )
}

fn get_hot_or_not_bet_details_for_this_post_impl_v1(
    cents_hot_or_not_game: CentsHotOrNotGame,
    current_time: &SystemTime,
    request_maker: &Principal,
    post_id: u64,
) -> BettingStatus {
    let post = cents_hot_or_not_game
        .canister_data
        .get_post(&post_id)
        .unwrap();

    post.get_hot_or_not_betting_status_for_this_post_v1(
        current_time,
        request_maker,
        &cents_hot_or_not_game
            .token_bet_game
            .hot_or_not_bet_details
            .room_details_map,
        &cents_hot_or_not_game
            .token_bet_game
            .hot_or_not_bet_details
            .post_principal_map,
        &cents_hot_or_not_game
            .token_bet_game
            .hot_or_not_bet_details
            .slot_details_map,
    )
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashSet,
        time::{Duration, SystemTime},
    };

    use shared_utils::{
        canister_specific::individual_user_template::types::{
            hot_or_not::HotOrNotDetails,
            post::{FeedScore, Post, PostViewStatistics},
        },
        common::types::top_posts::post_score_index_item::PostStatus,
    };

    use super::*;

    #[test]
    fn test_get_hot_or_not_bet_details_for_this_post_impl() {
        let mut canister_data = CanisterData::default();
        let post_id = 0;

        canister_data.add_post(Post {
            id: 0,
            is_nsfw: false,
            description: "Singing and dancing".to_string(),
            hashtags: vec!["sing".to_string(), "dance".to_string()],
            video_uid: "video#0001".to_string(),
            status: PostStatus::ReadyToView,
            created_at: SystemTime::now(),
            likes: HashSet::new(),
            share_count: 0,
            view_stats: PostViewStatistics::default(),
            home_feed_score: FeedScore::default(),
            hot_or_not_details: Some(HotOrNotDetails::default()),
            slots_left_to_be_computed: Default::default(),
        });

        let result = get_hot_or_not_bet_details_for_this_post_impl(
            &canister_data,
            &SystemTime::now(),
            &Principal::anonymous(),
            post_id,
        );
        match result {
            BettingStatus::BettingOpen { .. } => {}
            _ => panic!("Expected BettingStatus::BettingOpen"),
        }

        let result = get_hot_or_not_bet_details_for_this_post_impl(
            &canister_data,
            &SystemTime::now()
                .checked_add(Duration::from_secs(60 * 60 * 48 + 10))
                .unwrap(),
            &Principal::anonymous(),
            post_id,
        );
        match result {
            BettingStatus::BettingClosed => {}
            _ => panic!("Expected BettingStatus::BettingClosed"),
        }
    }
}
