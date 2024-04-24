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

#[query]
fn get_last_canister_functionality_access_time() -> Result<SystemTime, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .last_canister_functionality_access_time
            .ok_or(String::from("Canister has not been assigned yet"))
    })
}

pub fn last_canister_access_time() -> String {
    let last_canister_functionality_access_time = get_last_canister_functionality_access_time();

    match last_canister_functionality_access_time {
        Ok(time) => match serde_json::to_string(&time) {
            Ok(json) => json,
            Err(e) => e.to_string(),
        },
        Err(e) => e,
    }
}
