use candid::Principal;
use ic_cdk::{api::management_canister::provisional::CanisterId, update};
use ic_cdk_macros::query;
use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::BetOutcomeForBetMaker,
    canister_specific::individual_user_template::types::hot_or_not::{
        BetDetails, GlobalBetId, GlobalRoomId, PlacedBetDetail,
    },
    common::types::app_primitive_type::PostId,
    common::types::utility_token::token_event::TokenEvent,
};

use futures::StreamExt;
use shared_utils::canister_specific::individual_user_template::types::hot_or_not::StablePrincipal;
use shared_utils::common::types::utility_token::token_event::HotOrNotOutcomePayoutEvent;
use shared_utils::common::utils::system_time;
use std::collections::HashMap;

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[update]
pub async fn resolve_pending_bets() {
    update_last_canister_functionality_access_time();

    let user_principal_id =
        CANISTER_DATA.with_borrow(|canister_data| canister_data.profile.principal_id.clone());

    let user_principal_id = match user_principal_id {
        Some(id) => (id),
        None => return,
    };

    let pending_bets = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed
            .clone()
            .into_iter()
            .filter(|((_post_canister_id, _post_id), placed_bet_detail)| {
                matches!(
                    placed_bet_detail.outcome_received,
                    BetOutcomeForBetMaker::AwaitingResult
                )
            })
            .collect::<Vec<_>>()
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
                    *post_id,
                    placed_bet_detail.slot_id,
                    placed_bet_detail.room_id,
                ),
                StablePrincipal(user_principal_id),
            );
            call_caller_for_hot_or_not_bet_result(post_canister_id.clone(), global_bet_id.clone())
        });

    let stream = futures::stream::iter(futures).boxed().buffer_unordered(20);

    let results = stream
        .collect::<Vec<Result<(BetDetails, GlobalBetId, CanisterId), String>>>()
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
            let post_id = global_bet_id.0 .0;

            let current_time = system_time::get_current_system_time_from_ic();

            // get the pending_bet_details
            let pending_bet_details = pending_bets_hashmap
                .get(&(*post_canister_id, post_id))
                .unwrap();

            let mut pending_bet_details = pending_bet_details.clone();
            // update the pending_bet_details
            pending_bet_details.update_outcome(bet_details.clone().into());

            CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data.all_hot_or_not_bets_placed.insert(
                    (post_canister_id.clone(), post_id),
                    pending_bet_details.clone(),
                );

                let placed_bet_detail = pending_bet_details.clone();
                let outcome: BetOutcomeForBetMaker = placed_bet_detail.outcome_received;

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
                        post_canister_id: *post_canister_id,
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
) -> Result<(BetDetails, GlobalBetId, CanisterId), String> {
    let GlobalBetId(GlobalRoomId(post_canister_id, slot_id, room_id), caller) = global_bet_id;
    let StablePrincipal(caller_principal) = caller;
    let result = ic_cdk::call::<_, ()>(
        post_maker_canister_id,
        "get_bet_details_for_bet_id",
        ((post_canister_id, slot_id, room_id), caller_principal),
    )
    .await
    .map_err(|_| {
        "Could not call post maker canister for bet_details".into()
    })?.0?;

    match result {
        Ok(val) => match val {
            Some(bet_detail) => Ok((bet_detail, global_bet_id, post_maker_canister_id)),
            None => Err(format!("Bet Not Resolved yet for global_bet_id {:?}",global_bet_id)),
        },
        Err(e) => Err("Unknown Error while resolving pending bets".into()),
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
pub fn insert_in_bet_details_map(global_bet_id: GlobalBetId, bet_detail: BetDetails) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .bet_details_map
            .insert(global_bet_id, bet_detail);
    });
}
