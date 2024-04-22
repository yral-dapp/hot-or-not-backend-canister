use ic_cdk::api;
use ic_cdk_macros::query;

use crate::api::canister_management::update_last_access_time::update_last_canister_functionality_access_time;

#[query]
fn get_user_caniser_cycle_balance() -> u128 {
    api::canister_balance128()
}
