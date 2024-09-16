use candid::Principal;
use ic_cdk::api::management_canister::provisional::CanisterId;
use ic_cdk_macros::query;

use shared_utils::{
    canister_specific::individual_user_template::types::hot_or_not::PlacedBetDetail,
    common::types::app_primitive_type::PostId,
};
use shared_utils::canister_specific::individual_user_template::types::hot_or_not::{GlobalBetId, GlobalRoomId, StablePrincipal};
use shared_utils::canister_specific::individual_user_template::types::hot_or_not::BetDetails;
use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[query]
fn get_bet_details_for_bet_id(
    bet_id: ((u64,u8,u64),Principal)
) -> Option<BetDetails> {
    let ((post_canister_id,slot_id,room_id),caller) = bet_id;
    let bet_id_struct = GlobalBetId(GlobalRoomId(post_canister_id,slot_id,room_id),StablePrincipal(caller));
    update_last_canister_functionality_access_time();
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .bet_details_map
            .get(&bet_id_struct)
            // .cloned()
    })
}
