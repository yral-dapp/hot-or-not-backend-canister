use ic_cdk::api;
use ic_cdk_macros::query;

#[query]
fn get_cycle_balance() -> u128 {
    api::canister_balance128()
}
