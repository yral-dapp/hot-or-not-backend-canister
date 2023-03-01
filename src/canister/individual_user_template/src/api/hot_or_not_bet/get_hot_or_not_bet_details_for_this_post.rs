use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::BettingStatus,
    common::utils::system_time::{self, SystemTimeProvider},
};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk::query]
#[candid::candid_method(query)]
fn get_hot_or_not_bet_details_for_this_post(post_id: u64) -> BettingStatus {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        get_hot_or_not_bet_details_for_this_post_impl(
            &canister_data_ref_cell.borrow(),
            &system_time::get_current_system_time_from_ic,
            post_id,
        )
    })
}

fn get_hot_or_not_bet_details_for_this_post_impl(
    canister_data: &CanisterData,
    time_provider: &SystemTimeProvider,
    post_id: u64,
) -> BettingStatus {
    canister_data
        .all_created_posts
        .get(&post_id)
        .unwrap()
        .get_hot_or_not_betting_status_for_this_post(time_provider)
}

#[cfg(test)]
mod test {
    use std::{
        collections::HashSet,
        time::{Duration, SystemTime},
    };

    use shared_utils::{
        canister_specific::individual_user_template::types::post::{
            HotOrNotDetails, Post, PostViewStatistics,
        },
        types::canister_specific::individual_user_template::post::PostStatus,
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
                description: "Singing and dancing".to_string(),
                hashtags: vec!["sing".to_string(), "dance".to_string()],
                video_uid: "video#0001".to_string(),
                status: PostStatus::ReadyToView,
                created_at: SystemTime::now(),
                likes: HashSet::new(),
                share_count: 0,
                view_stats: PostViewStatistics::default(),
                homefeed_ranking_score: 0,
                creator_consent_for_inclusion_in_hot_or_not: true,
                hot_or_not_details: Some(HotOrNotDetails::default()),
            },
        );

        // TODO: flesh out this test
        let result = get_hot_or_not_bet_details_for_this_post_impl(
            &canister_data,
            &|| SystemTime::now(),
            post_id,
        );
        match result {
            BettingStatus::BettingOpen { .. } => {}
            _ => panic!("Expected BettingStatus::BettingOpen"),
        }

        let result = get_hot_or_not_bet_details_for_this_post_impl(
            &canister_data,
            &|| {
                SystemTime::now()
                    .checked_add(Duration::from_secs(60 * 60 * 48 + 10))
                    .unwrap()
            },
            post_id,
        );
        match result {
            BettingStatus::BettingClosed => {}
            _ => panic!("Expected BettingStatus::BettingClosed"),
        }
    }
}
