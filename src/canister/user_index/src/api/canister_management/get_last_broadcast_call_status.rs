use ic_cdk_macros::query;
use shared_utils::canister_specific::user_index::types::BroadcastCallStatus;

use crate::CANISTER_DATA;

#[query]
fn get_last_broadcast_call_status() -> BroadcastCallStatus {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.last_broadcast_call_status.clone())
}
