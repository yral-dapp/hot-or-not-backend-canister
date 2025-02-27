use ic_cdk_macros::query;

use crate::CANISTER_DATA;

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
