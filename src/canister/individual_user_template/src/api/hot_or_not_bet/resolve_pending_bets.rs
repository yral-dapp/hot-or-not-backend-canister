use candid::Principal;
use ic_cdk::{api::management_canister::provisional::CanisterId, update};
use ic_cdk_macros::query;
use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::{
        BetDetail, BetDetails, GlobalBetId, GlobalRoomId, PlacedBetDetail,
    },
    common::types::app_primitive_type::PostId,
    hot_or_not::BetOutcomeForBetMaker,
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};
use ic_cdk::caller;

#[update]
pub async fn resolve_pending_bets() {
    update_last_canister_functionality_access_time();

    let user_principal_id = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.profile.principal_id.cloned();
    });

    let pending_bets = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed
            .iter()
            .filter(|((_post_canister_id, _post_id), placed_bet_detail)| {
                matches!(
                    placed_bet_detail.outcome_received,
                    BetOutcomeForBetMaker::AwaitingResult
                )
            })
            .collect()
    });

    resolve_pending_bets_impl(pending_bets, user_principal_id).await;
}

async fn resolve_pending_bets_impl(
    pending_bets: Vec<((CanisterId, PostId), PlacedBetDetail)>,
    user_principal_id: Principal,
) {
    let futures = pending_bets
        .iter()
        .map(|((post_canister_id, post_id), placed_bet_detail)| {
            let global_bet_id = GlobalBetId(
                GlobalRoomId(
                    post_id,
                    placed_bet_detail.slot_id,
                    placed_bet_detail.room_id,
                ),
                user_principal_id,
            );
            call_caller_for_hot_or_not_bet_result(post_canister_id, global_bet_id)
        });

    let stream = futures::stream::iter(futures).boxed().buffer_unordered(20);

    let results = stream
        .collect::<Vec<Result<(BetDetails, GlobalBetId, CanisterId), Error>>>()
        .await;

    let success_bet_results = results
        .into_iter()
        .filter_map(|r| r.as_ref().ok().cloned())
        .collect::<Vec<(BetDetails, GlobalBetId, CanisterId)>>();

    let pending_bets_hashmap = pending_bets
        .into_iter()
        .collect::<HashMap<(CanisterId, PostId), PlacedBetDetail>>();

    // udpate all the pending bets with the result
    success_bet_results
        .iter()
        .for_each(|(bet_details, global_bet_id, post_canister_id)| {
            // get the pending_bet_details
            let pending_bet_details = pending_bets_hashmap
                .get(&(post_canister_id, global_bet_id.0 .0))
                .unwrap();

            // update the pending_bet_details
            pending_bet_details.outcome_received = bet_details.into();

            CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data.all_hot_or_not_bets_placed.insert(
                    (post_canister_id, global_bet_id.0 .0),
                    pending_bet_details.clone(),
                );

                // handle token balance
                // copied from: src/canister/individual_user_template/src/api/hot_or_not_bet/receive_bet_winnings_when_distributed.rs:58
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
        });
}

async fn call_caller_for_hot_or_not_bet_result(
    post_maker_canister_id: CanisterId,
    global_bet_id: GlobalBetId,
) -> Result<(BetDetails, GlobalBetId, CanisterId), Error> {
    let result = ic_cdk::call::<_, ()>(
        post_maker_canister_id,
        "get_bet_details_for_bet_id",
        (global_bet_id),
    )
    .await;

    match result {
        Ok(val) => match val {
            Some(bet_detail) => Ok((bet_detail, global_bet_id, post_canister_id)),
            None => Err(format!(
                "Bet Not Resolved yet for global_bet_id {:?}",
                global_bet_id
            )),
        },
        Err(e) => Err(e),
    }
}

// these two function scaffolds are for testing purposes ONLY
// they are used to simulate hung timers for integration tests

#[update]
pub fn insert_in_all_hot_or_not_bets_placed(
    post_canister_id: CanisterId,
    post_id: PostId,
    placed_bet_detail: PlacedBetDetail,
) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .all_hot_or_not_bets_placed
            .insert((post_canister_id, post_id), placed_bet_detail);
    });
}

#[update]
pub fn insert_in_bet_details_map(global_bet_id: GlobalBetId, bet_detail: BetDetail) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .bet_details_map
            .insert(global_bet_id, bet_detail);
    });
}
