use ic_cdk::query;
use shared_utils::canister_specific::user_index::types::RecycleStatus;

use crate::CANISTER_DATA;

#[query]
pub fn get_recycle_status() -> RecycleStatus {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.recycle_status.clone())
}
