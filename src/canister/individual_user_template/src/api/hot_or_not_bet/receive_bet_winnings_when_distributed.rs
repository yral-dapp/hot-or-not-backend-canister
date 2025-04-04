use ic_cdk_macros::update;

use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::{
        BetOutcomeForBetMaker, HotOrNotGame,
    },
    common::{types::app_primitive_type::PostId, utils::system_time},
};

use crate::{
    data_model::cents_hot_or_not_game::CentsHotOrNotGame,
    util::cycles::notify_to_recharge_canister, CANISTER_DATA, PUMP_N_DUMP,
};

#[update]
fn receive_bet_winnings_when_distributed(post_id: PostId, outcome: BetOutcomeForBetMaker) {
    notify_to_recharge_canister();

    let post_creator_canister_id = ic_cdk::caller();
    let current_time = system_time::get_current_system_time_from_ic();

    ic_cdk::println!(
        "Recieved bet outcome from canister {} for post {}",
        post_creator_canister_id.to_string(),
        post_id
    );

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.receive_earnings_for_the_bet(
            post_id,
            post_creator_canister_id,
            outcome,
            current_time,
        );
    })
}

#[update]
fn receive_bet_winnings_when_distributed_v1(post_id: PostId, outcome: BetOutcomeForBetMaker) {
    notify_to_recharge_canister();

    let post_creator_canister_id = ic_cdk::caller();
    let current_time = system_time::get_current_system_time_from_ic();

    ic_cdk::println!(
        "Recieved cents bet outcome from canister {} for post {}",
        post_creator_canister_id.to_string(),
        post_id
    );

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        PUMP_N_DUMP.with_borrow_mut(|token_bet_game| {
            let mut cents_hot_or_not_game = CentsHotOrNotGame {
                canister_data,
                token_bet_game,
            };
            cents_hot_or_not_game.receive_earnings_for_the_bet(
                post_id,
                post_creator_canister_id,
                outcome,
                current_time,
            );
        })
    })
}
