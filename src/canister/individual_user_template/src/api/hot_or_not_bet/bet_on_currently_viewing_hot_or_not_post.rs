use candid::Principal;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg, error::BetOnCurrentlyViewingPostError, hot_or_not::BettingStatus,
    },
    common::utils::system_time::{self, SystemTimeProvider},
};

use crate::{
    api::post::update_scores_and_share_with_post_cache_if_difference_beyond_threshold::update_scores_and_share_with_post_cache_if_difference_beyond_threshold,
    data_model::CanisterData, CANISTER_DATA,
};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn bet_on_currently_viewing_post(
    place_bet_arg: PlaceBetArg,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    let api_caller = ic_cdk::caller();

    if api_caller == Principal::anonymous() {
        return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
    }

    let response = CANISTER_DATA.with(|canister_data_ref_cell| {
        bet_on_currently_viewing_post_impl(
            &mut canister_data_ref_cell.borrow_mut(),
            &api_caller,
            place_bet_arg.clone(),
            &system_time::get_current_system_time_from_ic,
        )
    });

    if response.is_ok() {
        update_scores_and_share_with_post_cache_if_difference_beyond_threshold(
            place_bet_arg.post_id,
        );
    }

    response
}

fn bet_on_currently_viewing_post_impl(
    canister_data: &mut CanisterData,
    api_caller: &Principal,
    place_bet_arg: PlaceBetArg,
    time_provider: &SystemTimeProvider,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    let PlaceBetArg {
        post_id,
        bet_amount,
        bet_direction,
    } = place_bet_arg;

    if canister_data.my_token_balance.get_utility_token_balance() < bet_amount {
        return Err(BetOnCurrentlyViewingPostError::InsufficientBalance);
    }

    let post = canister_data.all_created_posts.get_mut(&post_id).unwrap();

    post.place_hot_or_not_bet(api_caller, bet_amount, &bet_direction, &time_provider())
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use shared_utils::canister_specific::individual_user_template::types::{
        hot_or_not::BetDirection,
        post::{Post, PostDetailsFromFrontend},
    };
    use test_utils::setup::test_constants::get_mock_user_alice_principal_id;

    use super::*;

    #[test]
    fn test_bet_on_currently_viewing_post_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.all_created_posts.insert(
            0,
            Post::new(
                0,
                PostDetailsFromFrontend {
                    description: "Doggos and puppers".into(),
                    hashtags: vec!["doggo".into(), "pupper".into()],
                    video_uid: "abcd#1234".into(),
                    creator_consent_for_inclusion_in_hot_or_not: true,
                },
                &SystemTime::now(),
            ),
        );

        let result = bet_on_currently_viewing_post_impl(
            &mut canister_data,
            &get_mock_user_alice_principal_id(),
            PlaceBetArg {
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
            &|| SystemTime::now(),
        );
        assert_eq!(
            result,
            Err(BetOnCurrentlyViewingPostError::InsufficientBalance)
        );

        canister_data.my_token_balance.utility_token_balance = 100;
        let result = bet_on_currently_viewing_post_impl(
            &mut canister_data,
            &get_mock_user_alice_principal_id(),
            PlaceBetArg {
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
            &|| SystemTime::now(),
        );

        let post = canister_data.all_created_posts.get(&0).unwrap();

        assert_eq!(
            result,
            Ok(BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 1,
                ongoing_slot: 1,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: Some(true)
            })
        );
    }
}
