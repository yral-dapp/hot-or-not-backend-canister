use std::cell::RefCell;

use candid::Principal;
use ic_cdk::api::call::CallResult;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::{BetMakerArg, PlaceBetArg},
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{
            BetOutcomeForBetMaker, BettingStatus, HotOrNotGame, HotOrNotGameV1, PlacedBetDetail,
        },
        pump_n_dump,
        token::{self, TokenTransactions},
    },
    common::{
        types::utility_token::token_event::{StakeEvent, TokenEvent},
        utils::{
            permissions::{is_caller_global_admin, is_caller_global_admin_v2},
            system_time,
        },
    },
};

use crate::{
    data_model::pump_n_dump::PumpAndDumpGame, util::cycles::notify_to_recharge_canister,
    CANISTER_DATA, PUMP_N_DUMP,
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
        let mut my_token_balance = canister_data.my_token_balance.clone();
        let result = HotOrNotGame::prepare_for_bet(
            canister_data,
            bet_maker_principal_id,
            &place_bet_arg,
            &mut my_token_balance,
            current_time,
        );
        canister_data.my_token_balance = my_token_balance;

        result
    })?;

    let call_result =
        call_post_maker_canister_to_place_bet(bet_maker_principal_id, &place_bet_arg).await;

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut my_token_balance = canister_data.my_token_balance.clone();
        let result = HotOrNotGame::process_place_bet_status(
            canister_data,
            call_result,
            &place_bet_arg,
            &mut my_token_balance,
            current_time,
        );

        canister_data.my_token_balance = my_token_balance;
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

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        PUMP_N_DUMP.with_borrow_mut(|pump_and_dump_game| {
            HotOrNotGameV1::prepare_for_bet(
                canister_data,
                bet_maker_principal_id,
                &place_bet_arg,
                pump_and_dump_game,
                current_time,
            )
        })
    })?;

    let call_result = call_post_maker_canister_to_place_bet_v1(
        &place_bet_arg,
        BetMakerArg {
            bet_maker_canister_id: ic_cdk::id(),
            bet_maker_principal_id: bet_maker_principal_id,
        },
    )
    .await;

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        PUMP_N_DUMP.with_borrow_mut(|pump_and_dump_game| {
            HotOrNotGameV1::process_place_bet_status(
                canister_data,
                call_result,
                &place_bet_arg,
                pump_and_dump_game,
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
    bet_maker_arg: BetMakerArg,
) -> CallResult<(Result<BettingStatus, BetOnCurrentlyViewingPostError>,)> {
    ic_cdk::call::<_, (Result<BettingStatus, BetOnCurrentlyViewingPostError>,)>(
        place_bet_arg.post_canister_id,
        "receive_bet_from_bet_makers_canister_v1",
        (place_bet_arg.clone(), bet_maker_arg),
    )
    .await
}
#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use shared_utils::canister_specific::individual_user_template::types::hot_or_not::BetDirection;
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id,
    };

    use crate::CanisterData;

    use super::*;

    #[test]
    fn test_validate_incoming_bet() {
        let mut canister_data = CanisterData::default();

        let result = HotOrNotGame::validate_incoming_bet(
            &canister_data,
            &canister_data.my_token_balance,
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

        let result = HotOrNotGame::validate_incoming_bet(
            &canister_data,
            &canister_data.my_token_balance,
            get_mock_user_bob_principal_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::Unauthorized));

        let result = HotOrNotGame::validate_incoming_bet(
            &canister_data,
            &canister_data.my_token_balance,
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

        let result = HotOrNotGame::validate_incoming_bet(
            &canister_data,
            &canister_data.my_token_balance,
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

        let result = HotOrNotGame::validate_incoming_bet(
            &canister_data,
            &canister_data.my_token_balance,
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
