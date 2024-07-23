use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BetOutcomeForBetMaker, BettingStatus, PlacedBetDetail},
    },
    common::{
        types::{app_primitive_type::PostId, utility_token::token_event::{StakeEvent, SystemTimeInMs, TokenEvent}},
        utils::system_time,
    },
};

use crate::api::hot_or_not_bet::tabulate_hot_or_not_outcome_for_post_slot::tabulate_hot_or_not_outcome_for_post_slot;
use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, CANISTER_DATA,
};

use ic_stable_structures::btreemap::BTreeMap;
use ic_stable_structures::{Memory, Storable};
use std::fmt::Debug;
use std::time::Duration;

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

    ic_cdk::println!(
        "{:?}",
        "calling response receive_bet_from_bet_makers_canister"
    );

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

    ic_cdk::println!("{:?}", &response);

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
                    ic_cdk::println!("current_time: {:?}", current_time);
                    canister_data.first_bet_placed_at_hashmap.insert(
                        place_bet_arg.post_id,
                        (SystemTimeInMs::from_system_time(current_time), ongoing_slot),
                    );
                    // also push the post_id to the queue
                    let bet_timer_posts = &mut canister_data.bet_timer_posts;
                    let _to_print = match bet_timer_posts.insert(
                        (
                            SystemTimeInMs::from_system_time(current_time),
                            place_bet_arg.post_id.clone(),
                        ),
                        (),
                    ) {
                        Some(timer) => format!("Timer pushed to empty array: {:?}", timer),
                        None => "Failed to push timer to empty array".to_string(),
                    };

                    ic_cdk::println!(
                        "before maybe_enqueue_timer - bet_timer_posts: {:?}",
                        print_btree_map(&canister_data.bet_timer_posts)
                    );

                    maybe_enqueue_timer(canister_data);
                }
                ic_cdk::println!("second bet on same post: {:?}", place_bet_arg.post_id);
            });
        }
    }

    Ok(response)
}

fn maybe_enqueue_timer(canister_data: &mut CanisterData) {
    let should_start_timer = match canister_data.is_timer_running {
        Some(post_id) => process_running_timer(canister_data, post_id),
        None => !canister_data.first_bet_placed_at_hashmap.is_empty(),
    };

    ic_cdk::println!(
        "inside maybe_enqueue_timer: timer_running = {:?} ; should_start_timer = {:?} \n",
        &canister_data.is_timer_running,
        should_start_timer
    );

    if should_start_timer {
        start_timer(canister_data);
    }
}

fn process_running_timer(canister_data: &mut CanisterData, post_id: u64) -> bool {
    ic_cdk::println!("process_running_timer for post_id: {:?}", post_id);
    if !timer_expired(post_id, canister_data) {
        // don't start timer again if previous one has not expired yet
        return false;
    }

    ic_cdk::println!("process_running_timer - canister_data.first_bet_placed_at_hashmap for post_id: {:?}",canister_data.first_bet_placed_at_hashmap.get(&post_id));

    if let Some((_, ongoing_slot)) = canister_data.first_bet_placed_at_hashmap.get(&post_id) {
        tabulate_hot_or_not_outcome_for_post_slot(canister_data, post_id, ongoing_slot);

        ic_cdk::println!(
            "\n\n canister_data.bet_timer_posts: {:?}",
            print_btree_map(&canister_data.bet_timer_posts)
        );
        ic_cdk::println!("\n\n BEFORE PROCESSING TIMER {}", "//".repeat(400));
        // remove the post_id from the hashmap and bet_timer_posts
        let val = canister_data.first_bet_placed_at_hashmap.remove(&post_id);
        let same_post_id = canister_data.bet_timer_posts.pop_first();

        canister_data.is_timer_running = None;
        ic_cdk::println!(" after PROCESSING TIMER {}", "--".repeat(400));
        ic_cdk::println!(
            "\n\n processed_timer -- post_id: {:?}, same_post_id: {:?}",
            post_id,
            same_post_id
        );
        ic_cdk::println!("\n\n canister_data.first_bet_placed_at_hashmap: {:?}", val);
        ic_cdk::println!(
            "\n\n canister_data.bet_timer_posts: {:?}",
            print_btree_map(&canister_data.bet_timer_posts)
        );
        ic_cdk::println!(" after PROCESSING TIMER {}", "--".repeat(400));

        // return true to indicate that timer has been processed
        true
    } else {
        false
    }
}

pub fn print_btree_map<K, V, M>(btree: &BTreeMap<K, V, M>) -> String
where
    K: Ord + Debug + Storable + Clone,
    V: Debug + Storable,
    M: Memory,
{
    let mut result = String::from("{");
    let mut iter = btree.iter();

    if let Some((key, value)) = iter.next() {
        result.push_str(&format!("{:?}: {:?}", key, value));
    }

    for (key, value) in iter {
        result.push_str(&format!(", {:?}: {:?}", key, value));
    }

    result.push('}');
    result
}

fn start_timer(canister_data: &mut CanisterData) {
    if !canister_data.first_bet_placed_at_hashmap.is_empty() {
        // bet_timer_posts is a queue with head at the last element of array
        // and tail at the first element of array.
        // this is because `pop` removes the last entry from the vec in ic_stable_structures
        if let Some(((insertion_time, first_post_id), _)) = canister_data.bet_timer_posts.first_key_value() {
            // if let Some(composite_key) = canister_data.bet_timer_posts.pop_first() {
            // ic_cdk::println!("\n--------- post_id: {}, starting timer ------\n", first_post_id);
            ic_cdk::println!("\n--------- post_id: {:?}, starting timer ------\n", first_post_id);
            if let Some((bet_placed_time, _ongoing_slot_for_post)) =
                canister_data.first_bet_placed_at_hashmap.get(&first_post_id)
            {
                let current_time = SystemTimeInMs::now();
                let interval = current_time.duration_since(&bet_placed_time);

                canister_data.is_timer_running = Some(first_post_id);

                ic_cdk_timers::set_timer(interval, move || {

                    CANISTER_DATA.with(|canister_data_ref_cell| {
                        let canister_data = &mut canister_data_ref_cell.borrow_mut();
                        ic_cdk::println!("\n--------- TIME DONE for post id: ---------\n");

                        maybe_enqueue_timer(canister_data);
                    });
                });
            }
        }
    }
}

fn timer_expired(post_id: PostId, canister_data: &CanisterData) -> bool {
    if !canister_data.first_bet_placed_at_hashmap.is_empty() {
        if let Some(((first_time, first_post_id), _)) =
            canister_data.bet_timer_posts.first_key_value()
        {
            if let Some((bet_placed_time, _ongoing_slot_for_post)) =
                canister_data.first_bet_placed_at_hashmap.get(&post_id)
            {
                let current_time = SystemTimeInMs::now();
                let interval = current_time.duration_since(&bet_placed_time);
                let return_val = interval > TIMER_DURATION;

                ic_cdk::println!(
                    "post_id ({}) == last_post_id ({}), {}, \n bet_timer_posts: {:?}, \n bet_timer_posts.time: {:?} === first_bet_placed_at_hashmap.time: {:?}", 
                    post_id,
                    first_post_id,
                    post_id == first_post_id,
                    print_btree_map(&canister_data.bet_timer_posts),
                    first_time, 
                    bet_placed_time
                );

                ic_cdk::println!("post_id: {}, timer_expired: {:?}", post_id, return_val);

                return return_val;
            }
        }
    }
    ic_cdk::println!(
        "post_id: {}, timer_expired: FALSE (returning from outside)",
        post_id
    );

    false
}

// fn timer_expired(post_id: PostId, canister_data: &CanisterData) -> bool {
//     if !canister_data.first_bet_placed_at_hashmap.is_empty() {
//         let last_post_index = canister_data.bet_timer_posts.len() - 1;
//         if let Some(last_post_id) = canister_data.bet_timer_posts.get(last_post_index) {
//             if let Some((bet_placed_time, _ongoing_slot_for_post)) =
//                 canister_data.first_bet_placed_at_hashmap.get(&post_id)
//             {
//                 let current_time = SystemTimeInMs::now();
//                 let interval = current_time.duration_since(&bet_placed_time);
//                 let return_val = interval > TIMER_DURATION;

//                 ic_cdk::println!(
//                     "post_id == last_post_id : {}, {}, {}, \n bet_timer_posts: {:?}, \n last_post_index: {:?}",
//                     post_id,
//                     last_post_id,
//                     post_id == last_post_id,
//                     canister_data.bet_timer_posts,
//                     last_post_index
//                 );

//                 ic_cdk::println!("post_id: {}, timer_expired: {:?}", post_id, return_val);

//                 return return_val;
//             }
//         }
//     }
//     ic_cdk::println!(
//         "post_id: {}, timer_expired: FALSE (returning from outside)",
//         post_id
//     );

//     false
// }

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
