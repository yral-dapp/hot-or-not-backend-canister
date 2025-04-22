use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk_macros::query;

use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::PlacedBetDetail,
    common::types::app_primitive_type::PostId,
};

use crate::{CANISTER_DATA, PUMP_N_DUMP};

#[deprecated]
#[query]
fn get_individual_hot_or_not_bet_placed_by_this_profile(
    canister_id: CanisterId,
    post_id: PostId,
) -> Option<PlacedBetDetail> {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .all_hot_or_not_bets_placed
            .get(&(canister_id, post_id))
            .cloned()
    })
}

#[query]
fn get_individual_hot_or_not_bet_placed_by_this_profile_v1(
    canister_id: CanisterId,
    post_id: PostId,
) -> Option<PlacedBetDetail> {
    PUMP_N_DUMP.with_borrow(|token_bet_game| {
        token_bet_game
            .hot_or_not_bet_details
            .all_hot_or_not_bets_placed
            .get(&(canister_id, post_id))
            .cloned()
    })
}
