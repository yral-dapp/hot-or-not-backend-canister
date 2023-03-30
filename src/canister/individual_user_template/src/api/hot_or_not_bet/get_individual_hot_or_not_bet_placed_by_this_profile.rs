use ic_cdk::api::management_canister::provisional::CanisterId;
use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::PlacedBetDetail,
    common::types::app_primitive_type::PostId,
};

use crate::CANISTER_DATA;

#[ic_cdk::query]
#[candid::candid_method(query)]
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
