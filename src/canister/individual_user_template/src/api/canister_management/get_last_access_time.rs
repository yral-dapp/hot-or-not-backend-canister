use std::time::SystemTime;

use ic_cdk_macros::query;

use crate::CANISTER_DATA;

#[query]
fn get_last_access_time() -> Result<SystemTime, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .last_access_time
            .ok_or(String::from("Canister has not been assigned yet"))
    })
}
