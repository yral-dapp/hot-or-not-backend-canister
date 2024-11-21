use ic_cdk_macros::update;

use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::BetOutcomeForBetMaker,
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
    CANISTER_DATA,
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

    if !CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed
            .contains_key(&(post_creator_canister_id, post_id))
    }) {
        return;
    }

    if !CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed
            .get(&(post_creator_canister_id, post_id))
            .unwrap()
            .outcome_received
            == BetOutcomeForBetMaker::AwaitingResult
    }) {
        return;
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        let all_hot_or_not_bets_placed = &mut canister_data.all_hot_or_not_bets_placed;

        all_hot_or_not_bets_placed
            .entry((post_creator_canister_id, post_id))
            .and_modify(|placed_bet_detail| {
                placed_bet_detail.outcome_received = outcome.clone();
            });

        let placed_bet_detail = all_hot_or_not_bets_placed
            .get(&(post_creator_canister_id, post_id))
            .cloned()
            .unwrap();

        let my_token_balance = &mut canister_data.my_token_balance;
        my_token_balance.handle_token_event(TokenEvent::HotOrNotOutcomePayout {
            amount: match outcome {
                BetOutcomeForBetMaker::Draw(amount) => amount,
                BetOutcomeForBetMaker::Won(amount) => amount,
                _ => 0,
            },
            details: HotOrNotOutcomePayoutEvent::WinningsEarnedFromBet {
                post_canister_id: post_creator_canister_id,
                post_id,
                slot_id: placed_bet_detail.slot_id,
                room_id: placed_bet_detail.room_id,
                winnings_amount: match outcome {
                    BetOutcomeForBetMaker::Draw(amount) => amount,
                    BetOutcomeForBetMaker::Won(amount) => amount,
                    _ => 0,
                },
                event_outcome: outcome,
            },
            timestamp: current_time,
        });
    });
}
