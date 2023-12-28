use ic_cdk::api::stable::stable_size;

use crate::CANISTER_DATA;

#[candid::candid_method(query)]
#[ic_cdk::query]
pub fn get_stable_memory_size() -> u32 {
    stable_size()
}


#[candid::candid_method(query)]
#[ic_cdk::query]
pub fn get_version_number() -> u64 {
    CANISTER_DATA.with(|canister_data_ref| {
        canister_data_ref.borrow().version_details.version_number
    })
}