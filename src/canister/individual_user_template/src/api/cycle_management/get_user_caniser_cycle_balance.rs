use ic_cdk::api;
use ic_cdk_macros::query;

#[query]
fn get_user_caniser_cycle_balance() -> u128 {
    api::canister_balance128()
}
