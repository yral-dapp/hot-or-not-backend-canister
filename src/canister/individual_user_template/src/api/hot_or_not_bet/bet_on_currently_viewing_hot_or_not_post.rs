use candid::Principal;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BettingStatus, PlacedBetDetail},
    },
    common::{
        types::utility_token::token_event::{StakeEvent, TokenEvent},
        utils::system_time,
    },
};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
// #[candid::candid_method(update)]
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

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = &mut canister_data_ref_cell.borrow_mut();

        let my_token_balance = &mut canister_data.my_token_balance;
        my_token_balance.handle_token_event(TokenEvent::Stake {
            details: StakeEvent::BetOnHotOrNotPost {
                post_canister_id: place_bet_arg.post_canister_id,
                post_id: place_bet_arg.post_id,
                bet_amount: place_bet_arg.bet_amount,
                bet_direction: place_bet_arg.bet_direction,
            },
            timestamp: current_time,
        });

        let all_hot_or_not_bets_placed = &mut canister_data.all_hot_or_not_bets_placed;
        all_hot_or_not_bets_placed.insert(PlacedBetDetail {
            canister_id: place_bet_arg.post_canister_id,
            post_id: place_bet_arg.post_id,
        });
    });

    Ok(response)
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
        .contains(&PlacedBetDetail {
            canister_id: place_bet_arg.post_canister_id,
            post_id: place_bet_arg.post_id,
        })
    {
        return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use shared_utils::canister_specific::individual_user_template::types::hot_or_not::BetDirection;
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

        canister_data
            .all_hot_or_not_bets_placed
            .insert(PlacedBetDetail {
                canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
            });

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
