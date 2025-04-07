use std::cell::RefCell;

use candid::Principal;
use ic_cdk::api::call::CallResult;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BettingStatus, HotOrNotGame},
    },
    common::utils::{permissions::is_caller_global_admin, system_time},
};

use crate::{
    data_model::cents_hot_or_not_game::CentsHotOrNotGame,
    util::cycles::notify_to_recharge_canister, CANISTER_DATA, PUMP_N_DUMP,
};

#[deprecated(note = "use bet_on_currently_viewing_post_v1")]
#[update]
async fn bet_on_currently_viewing_post(
    place_bet_arg: PlaceBetArg,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    notify_to_recharge_canister();

    let bet_maker_principal_id = ic_cdk::caller();
    let current_time = system_time::get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let result =
            canister_data.prepare_for_bet(bet_maker_principal_id, &place_bet_arg, current_time);

        result
    })?;

    let call_result =
        call_post_maker_canister_to_place_bet(bet_maker_principal_id, &place_bet_arg).await;

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let result =
            canister_data.process_place_bet_status(call_result, &place_bet_arg, current_time);

        result
    })
}

#[update(guard = "is_caller_global_admin")]
async fn bet_on_currently_viewing_post_v1(
    place_bet_arg: PlaceBetArg,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    notify_to_recharge_canister();

    let bet_maker_principal_id = CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .profile
            .principal_id
            .ok_or(BetOnCurrentlyViewingPostError::UserPrincipalNotSet)
    })?;
    let current_time = system_time::get_current_system_time_from_ic();

    PUMP_N_DUMP.with_borrow_mut(|token_bet_game| {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            let mut cents_hot_or_not_game = CentsHotOrNotGame {
                canister_data,
                token_bet_game,
            };

            cents_hot_or_not_game.prepare_for_bet(
                bet_maker_principal_id,
                &place_bet_arg,
                current_time,
            )
        })
    })?;

    let call_result =
        call_post_maker_canister_to_place_bet_v1(&place_bet_arg, bet_maker_principal_id).await;

    PUMP_N_DUMP.with_borrow_mut(|token_bet_game| {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            let mut cents_hot_or_not_game = CentsHotOrNotGame {
                canister_data,
                token_bet_game,
            };

            cents_hot_or_not_game.process_place_bet_status(
                call_result,
                &place_bet_arg,
                current_time,
            )
        })
    })
}

#[deprecated(note = "use call_post_maker_canister_to_place_bet_v1")]
async fn call_post_maker_canister_to_place_bet(
    bet_maker_principal: Principal,
    place_bet_arg: &PlaceBetArg,
) -> CallResult<(Result<BettingStatus, BetOnCurrentlyViewingPostError>,)> {
    ic_cdk::call::<_, (Result<BettingStatus, BetOnCurrentlyViewingPostError>,)>(
        place_bet_arg.post_canister_id,
        "receive_bet_from_bet_makers_canister",
        (place_bet_arg.clone(), bet_maker_principal),
    )
    .await
}

async fn call_post_maker_canister_to_place_bet_v1(
    place_bet_arg: &PlaceBetArg,
    bet_maker_principal: Principal,
) -> CallResult<(Result<BettingStatus, BetOnCurrentlyViewingPostError>,)> {
    ic_cdk::call::<_, (Result<BettingStatus, BetOnCurrentlyViewingPostError>,)>(
        place_bet_arg.post_canister_id,
        "receive_bet_from_bet_makers_canister_v1",
        (place_bet_arg.clone(), bet_maker_principal),
    )
    .await
}
#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use shared_utils::canister_specific::individual_user_template::types::hot_or_not::{
        BetDirection, BetOutcomeForBetMaker, PlacedBetDetail,
    };
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id,
    };

    use crate::CanisterData;

    use super::*;

    #[test]
    fn test_validate_incoming_bet() {
        let mut canister_data = CanisterData::default();

        let result = canister_data.validate_incoming_bet(
            Principal::anonymous(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn));

        canister_data.profile.principal_id = Some(get_mock_user_alice_principal_id());

        let result = canister_data.validate_incoming_bet(
            get_mock_user_bob_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::Unauthorized));

        let result = canister_data.validate_incoming_bet(
            get_mock_user_alice_principal_id(),
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

        let result = canister_data.validate_incoming_bet(
            get_mock_user_alice_principal_id(),
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
                slot_id: 1,
                room_id: 1,
                amount_bet: 100,
                bet_direction: BetDirection::Hot,
                bet_placed_at: SystemTime::now(),
                outcome_received: BetOutcomeForBetMaker::default(),
            },
        );

        let result = canister_data.validate_incoming_bet(
            get_mock_user_alice_principal_id(),
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
