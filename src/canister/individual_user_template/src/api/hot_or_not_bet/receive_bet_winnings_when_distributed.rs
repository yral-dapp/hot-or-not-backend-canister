use ic_cdk_macros::update;

use shared_utils::{
    canister_specific::individual_user_template::types::{
        hot_or_not::{BetOutcomeForBetMaker, HotOrNotGame, HotOrNotGameV1},
        token::TokenTransactions,
    },
    common::{
        types::{
            app_primitive_type::PostId,
            utility_token::token_event::{HotOrNotOutcomePayoutEvent, TokenEvent},
        },
        utils::system_time,
    },
};

use crate::{
    util::cycles::{notify_to_recharge_canister, recharge_canister},
    CANISTER_DATA, PUMP_N_DUMP,
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
        let mut utility_token = canister_data.my_token_balance.clone();
        HotOrNotGame::receive_earnings_for_the_bet(
            canister_data,
            post_id,
            post_creator_canister_id,
            outcome,
            &mut utility_token,
            current_time,
        );
        canister_data.my_token_balance = utility_token;
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
        PUMP_N_DUMP.with_borrow_mut(|pump_and_dump| {
            HotOrNotGameV1::receive_earnings_for_the_bet(
                canister_data,
                post_id,
                post_creator_canister_id,
                outcome,
                pump_and_dump,
                current_time,
            );
        })
    })
}
