use ic_cdk::api::stable::stable_size;
use ic_cdk_macros::query;

use crate::CANISTER_DATA;

pub mod get_last_access_time;
pub mod get_session_type;
pub mod update_last_access_time;
pub mod update_profile_owner;
pub mod update_session_type;

#[query]
pub fn get_stable_memory_size() -> u32 {
    stable_size()
}

#[query]
pub fn get_version_number() -> u64 {
    CANISTER_DATA
        .with(|canister_data_ref| canister_data_ref.borrow().version_details.version_number)
}

#[query]
pub fn get_version() -> String {
    CANISTER_DATA
        .with(|canister_data_ref| canister_data_ref.borrow().version_details.version.clone())
}
