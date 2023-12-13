use std::time::SystemTime;

use candid::Principal;
use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::BettingStatus,
    common::utils::system_time::{self},
};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk::query]
#[candid::candid_method(query)]
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

fn get_hot_or_not_bet_details_for_this_post_impl(
    canister_data: &CanisterData,
    current_time: &SystemTime,
    request_maker: &Principal,
    post_id: u64,
) -> BettingStatus {
    canister_data
        .all_created_posts
        .get(&post_id)
        .unwrap()
        .get_hot_or_not_betting_status_for_this_post(current_time, request_maker)
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashSet,
        time::{Duration, SystemTime},
    };

    use shared_utils::canister_specific::individual_user_template::types::{
        hot_or_not::HotOrNotDetails,
        post::{FeedScore, Post, PostStatus, PostViewStatistics},
    };

    use super::*;

    #[test]
    fn test_get_hot_or_not_bet_details_for_this_post_impl() {
        let mut canister_data = CanisterData::default();
        let post_id = 0;

        canister_data.all_created_posts.insert(
            0,
            Post {
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
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: Some(HotOrNotDetails::default()),
            },
        );

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
