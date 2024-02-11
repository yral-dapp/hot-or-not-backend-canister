use candid::candid_method;
use ic_cdk::caller;

use crate::CANISTER_DATA;


#[ic_cdk::update]
#[candid_method(update)]
pub fn subnet_orchestrator_maxed_out() {
    let subnet_orchestrator_canister_id = caller();
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.subet_orchestrator_with_capacity_left.remove(&subnet_orchestrator_canister_id);
    });
}