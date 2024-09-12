use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk_macros::query;

use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::PlacedBetDetail,
    common::types::app_primitive_type::PostId,
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[query]
fn get_bet_details_for_bet_id(
    bet_id: GlobalBetId
) -> Option<BetDetails> {
    update_last_canister_functionality_access_time();
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .bet_details_map
            .get(&bet_id)
            .cloned()
    })
}
