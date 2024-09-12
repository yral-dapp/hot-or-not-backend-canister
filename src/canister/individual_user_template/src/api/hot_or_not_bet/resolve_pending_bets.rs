use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk_macros::query;

use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::{
        BetDetail, BetDetails, GlobalBetId, GlobalRoomId, PlacedBetDetail,
    },
    common::types::app_primitive_type::PostId,
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
            .filter(|((post_canister_id, post_id), placed_bet_detail)| {
                matches!(
                    placed_bet_detail.outcome_received,
                    BetOutcomeForBetMaker::AwaitingResult
                )
            })
            .collect()
    });

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

    let result_callback = |res| {
        if let Ok(bet_details) = res {
            CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data
                    .all_hot_or_not_bets_placed
                    .get((post_canister_id, post_id))
                    .map(|placed_bet_detail|{
                        placed_bet_detail.outcome_received = bet_details.into();
                    })
            })
        }
    };

    let stream = futures::stream::iter(futures).boxed().buffer_unordered(20);

    let _ = stream.collect::<Vec<()>>().await;
}

async fn call_caller_for_hot_or_not_bet_result(
    post_maker_canister_id: CanisterId,
    global_bet_id: GlobalBetId,
) -> Result<BetDetails, Error> {
    let result = ic_cdk::call::<_, ()>(
        post_maker_canister_id,
        "get_bet_details_for_bet_id",
        (global_bet_id),
    )
    .await;

    match result {
        Ok(val) => match val {
            Some(bet_detail) => Ok(bet_detail),
            None => Err("Bet Not Resolved yet".to_string()),
        },
        Err(e) => Err(e),
    }
}
