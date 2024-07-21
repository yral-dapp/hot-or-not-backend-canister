use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BetOutcomeForBetMaker, BettingStatus, PlacedBetDetail},
    },
    common::{
        types::app_primitive_type::PostId,
        types::utility_token::token_event::{StakeEvent, SystemTimeInMs, TokenEvent},
        utils::system_time,
    },
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, CANISTER_DATA,
};
use crate::api::hot_or_not_bet::tabulate_hot_or_not_outcome_for_post_slot::tabulate_hot_or_not_outcome_for_post_slot;

use std::time::{Duration, SystemTime};

const TIMER_DURATION: Duration = Duration::from_secs(60 * 60); // 60 minutes

#[update]
async fn bet_on_currently_viewing_post(
    place_bet_arg: PlaceBetArg,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    let bet_maker_principal_id = ic_cdk::caller();
    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        validate_incoming_bet(
            &canister_data_ref_cell.borrow(),
            &bet_maker_principal_id,
            &place_bet_arg,
        )
    })?;

    update_last_canister_functionality_access_time();

    let response = ic_cdk::call::<_, (Result<BettingStatus, BetOnCurrentlyViewingPostError>,)>(
        place_bet_arg.post_canister_id,
        "receive_bet_from_bet_makers_canister",
        (
            place_bet_arg.clone(),
            CANISTER_DATA.with(|canister_data_ref_cell| {
                canister_data_ref_cell
                    .borrow()
                    .profile
                    .principal_id
                    .unwrap()
            }),
        ),
    )
    .await
    .map_err(|_| BetOnCurrentlyViewingPostError::PostCreatorCanisterCallFailed)?
    .0?;

    match response {
        // this case should never match in yral game implementation
        BettingStatus::BettingClosed => {
            return Err(BetOnCurrentlyViewingPostError::BettingClosed);
        }
        BettingStatus::BettingOpen {
            ongoing_slot,
            ongoing_room,
            ..
        } => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                let canister_data = &mut canister_data_ref_cell.borrow_mut();

                let my_token_balance = &mut canister_data.my_token_balance;
                my_token_balance.handle_token_event(TokenEvent::Stake {
                    amount: place_bet_arg.bet_amount,
                    details: StakeEvent::BetOnHotOrNotPost {
                        post_canister_id: place_bet_arg.post_canister_id,
                        post_id: place_bet_arg.post_id,
                        bet_amount: place_bet_arg.bet_amount,
                        bet_direction: place_bet_arg.bet_direction.clone(),
                    },
                    timestamp: current_time,
                });

                let all_hot_or_not_bets_placed = &mut canister_data.all_hot_or_not_bets_placed;
                all_hot_or_not_bets_placed.insert(
                    (place_bet_arg.post_canister_id, place_bet_arg.post_id),
                    PlacedBetDetail {
                        canister_id: place_bet_arg.post_canister_id,
                        post_id: place_bet_arg.post_id,
                        slot_id: ongoing_slot,
                        room_id: ongoing_room,
                        bet_direction: place_bet_arg.bet_direction,
                        bet_placed_at: current_time,
                        amount_bet: place_bet_arg.bet_amount,
                        outcome_received: BetOutcomeForBetMaker::default(),
                    },
                );

                // insert only the first bet in first_bet_placed_at_hashmap
                if !canister_data
                    .first_bet_placed_at_hashmap
                    .contains_key(&place_bet_arg.post_id)
                {
                    canister_data.first_bet_placed_at_hashmap.insert(
                        place_bet_arg.post_id,
                        (SystemTimeInMs::from_system_time(current_time), ongoing_slot),
                    );
                    // also push the post_id to the array
                    let bet_timer_posts = &mut canister_data.bet_timer_posts;
                    if bet_timer_posts.is_empty() {
                        let to_print = match bet_timer_posts.push(&place_bet_arg.post_id) {
                            Ok(timer) => format!("Timer pushed to empty array: {:?}", timer),
                            Err(_) => "Failed to push timer to empty array".to_string(),
                        };

                        ic_cdk::println!("{}", to_print);
                    } else {
                        bet_timer_posts.set(0, &place_bet_arg.post_id);
                    };

                    maybe_enqueue_timer();
                }
            });
        }
    }

    Ok(response)
}

fn maybe_enqueue_timer() {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = &mut canister_data_ref_cell.borrow_mut();

        match canister_data.is_timer_running {
            None => {
                if !canister_data.first_bet_placed_at_hashmap.is_empty() {
                    start_timer(canister_data);
                }
            }
            Some(post_id) => {
                if timer_expired(post_id, &canister_data) {
                    if let Some((_bet_placed_time, ongoing_slot_for_post)) = canister_data.first_bet_placed_at_hashmap.get(&post_id){ 
                    tabulate_hot_or_not_outcome_for_post_slot(canister_data, post_id, ongoing_slot_for_post);
                    start_timer(canister_data);
                }
            };
            }
        }
    });
}
fn start_timer(canister_data: &mut CanisterData) {
    if !canister_data.first_bet_placed_at_hashmap.is_empty() {
        // bet_timer_posts is a queue with head at the last element of array
        // and tail at the first element of array.
        // this is because `pop` removes the last entry from the vec in ic_stable_structures
        let post_id = canister_data.bet_timer_posts.pop().unwrap();

        if let Some((bet_placed_time, _ongoing_slot_for_post)) = canister_data.first_bet_placed_at_hashmap.get(&post_id) { 
            let current_time = SystemTime::now();
            let interval = current_time
                .duration_since(bet_placed_time.to_system_time().unwrap())
                .unwrap_or(Duration::ZERO);

            canister_data.is_timer_running = Some(post_id);

            ic_cdk_timers::set_timer(interval, move || {
                maybe_enqueue_timer();
            });
        }
    }
}

fn timer_expired(post_id: PostId, canister_data: &CanisterData) -> bool {
    if !canister_data.first_bet_placed_at_hashmap.is_empty() {
        let last_post_index = canister_data.bet_timer_posts.len() - 1;
        let last_post_id = canister_data.bet_timer_posts.get(last_post_index).unwrap();
        // if post_id == last_post_id {
        ic_cdk::println!(
            "post_id == last_post_id : {}, {}, {}",
            post_id,
            last_post_id,
            post_id == last_post_id
        );
        if let Some((bet_placed_time, _ongoing_slot_for_post)) = canister_data.first_bet_placed_at_hashmap.get(&post_id) { 
            let current_time = SystemTime::now();
            let interval = current_time
                .duration_since(bet_placed_time.to_system_time().unwrap())
                .unwrap_or(Duration::ZERO);
            return interval > TIMER_DURATION;
        }
        // }
    }
    false
}

fn validate_incoming_bet(
    canister_data: &CanisterData,
    bet_maker_principal_id: &Principal,
    place_bet_arg: &PlaceBetArg,
) -> Result<(), BetOnCurrentlyViewingPostError> {
    if *bet_maker_principal_id == Principal::anonymous() {
        return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
    }

    let profile_owner = canister_data
        .profile
        .principal_id
        .ok_or(BetOnCurrentlyViewingPostError::UserPrincipalNotSet)?;

    if *bet_maker_principal_id != profile_owner {
        return Err(BetOnCurrentlyViewingPostError::Unauthorized);
    }

    let utlility_token_balance = canister_data.my_token_balance.get_utility_token_balance();

    if utlility_token_balance < place_bet_arg.bet_amount {
        return Err(BetOnCurrentlyViewingPostError::InsufficientBalance);
    }

    if canister_data
        .all_hot_or_not_bets_placed
        .contains_key(&(place_bet_arg.post_canister_id, place_bet_arg.post_id))
    {
        return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use shared_utils::{
        canister_specific::individual_user_template::types::hot_or_not::BetDirection,
        common::types::utility_token::token_event::NewSlotType,
    };
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_validate_incoming_bet() {
        let mut canister_data = CanisterData::default();

        let result = validate_incoming_bet(
            &canister_data,
            &Principal::anonymous(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn));

        canister_data.profile.principal_id = Some(get_mock_user_alice_principal_id());

        let result = validate_incoming_bet(
            &canister_data,
            &get_mock_user_bob_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::Unauthorized));

        let result = validate_incoming_bet(
            &canister_data,
            &get_mock_user_alice_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(
            result,
            Err(BetOnCurrentlyViewingPostError::InsufficientBalance)
        );

        canister_data.my_token_balance.utility_token_balance = 1000;

        let result = validate_incoming_bet(
            &canister_data,
            &get_mock_user_alice_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Ok(()));

        canister_data.all_hot_or_not_bets_placed.insert(
            (get_mock_user_alice_canister_id(), 0),
            PlacedBetDetail {
                canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                slot_id: NewSlotType(1),
                room_id: 1,
                amount_bet: 100,
                bet_direction: BetDirection::Hot,
                bet_placed_at: SystemTime::now(),
                outcome_received: BetOutcomeForBetMaker::default(),
            },
        );

        let result = validate_incoming_bet(
            &canister_data,
            &get_mock_user_alice_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(
            result,
            Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost)
        );
    }
}
