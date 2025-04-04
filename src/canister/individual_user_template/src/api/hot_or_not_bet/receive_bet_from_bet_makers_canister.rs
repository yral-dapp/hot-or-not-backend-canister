use ic_cdk_macros::update;

use candid::Principal;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        arg::PlaceBetArg,
        error::BetOnCurrentlyViewingPostError,
        hot_or_not::{BettingStatus, HotOrNotGame},
    },
    common::utils::system_time::get_current_system_time,
};

use crate::{
    data_model::cents_hot_or_not_game::CentsHotOrNotGame,
    util::cycles::notify_to_recharge_canister, CANISTER_DATA, PUMP_N_DUMP,
};

#[deprecated(note = "use receive_bet_from_bet_makers_canister_v2")]
#[update]
fn receive_bet_from_bet_makers_canister(
    place_bet_arg: PlaceBetArg,
    bet_maker_principal_id: Principal,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    notify_to_recharge_canister();

    let current_timestamp = get_current_system_time();
    let bet_maker_canister_id = ic_cdk::caller();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.receive_bet_from_bet_maker_canister(
            bet_maker_principal_id,
            bet_maker_canister_id,
            &place_bet_arg,
            current_timestamp,
        )
    })
}

#[update]
fn receive_bet_from_bet_makers_canister_v1(
    place_bet_arg: PlaceBetArg,
    bet_maker_principal_id: Principal,
) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
    notify_to_recharge_canister();

    let bet_maker_canister_id = ic_cdk::caller();
    let current_timestamp = get_current_system_time();

    PUMP_N_DUMP.with_borrow_mut(|token_bet_game| {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            let mut cents_hot_or_not_game = CentsHotOrNotGame {
                canister_data,
                token_bet_game,
            };
            cents_hot_or_not_game.receive_bet_from_bet_maker_canister(
                bet_maker_principal_id,
                bet_maker_canister_id,
                &place_bet_arg,
                current_timestamp,
            )
        })
    })
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use shared_utils::canister_specific::individual_user_template::types::{
        hot_or_not::{BetDirection, GlobalBetId, GlobalRoomId, StablePrincipal},
        post::{Post, PostDetailsFromFrontend},
    };
    use test_utils::setup::test_constants::{
        get_mock_user_alice_canister_id, get_mock_user_alice_principal_id,
    };

    use crate::CanisterData;

    use super::*;

    #[test]
    fn test_receive_bet_from_bet_makers_canister_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.add_post(Post::new(
            0,
            &PostDetailsFromFrontend {
                is_nsfw: false,
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        ));

        let result = HotOrNotGame::receive_bet_from_bet_maker_canister(
            &mut canister_data,
            get_mock_user_alice_principal_id(),
            get_mock_user_alice_canister_id(),
            &PlaceBetArg {
                post_canister_id: get_mock_user_alice_canister_id(),
                post_id: 0,
                bet_amount: 100,
                bet_direction: BetDirection::Hot,
            },
            SystemTime::now(),
        );

        let post = canister_data.get_post(&0).unwrap();
        let global_room_id = GlobalRoomId(0, 1, 1);
        let global_bet_id = GlobalBetId(
            global_room_id,
            StablePrincipal(get_mock_user_alice_principal_id()),
        );

        let room_details = canister_data.room_details_map.get(&global_room_id).unwrap();
        let bet_details = canister_data.bet_details_map.get(&global_bet_id).unwrap();

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

        assert_eq!(room_details.room_bets_total_pot, 100);
        assert_eq!(room_details.total_hot_bets, 1);
        assert_eq!(room_details.total_not_bets, 0);

        assert_eq!(bet_details.amount, 100);
        assert_eq!(bet_details.bet_direction, BetDirection::Hot);
        assert_eq!(
            bet_details.bet_maker_canister_id,
            get_mock_user_alice_canister_id()
        );
    }
}
