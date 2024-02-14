use candid::Principal;
use ic_cdk_macros::query;

use crate::CANISTER_DATA;



#[query]
fn get_next_available_subnet() -> Principal {
    CANISTER_DATA.with_borrow(|canister_data| {
        *canister_data.subet_orchestrator_with_capacity_left.iter().next().unwrap()
    })
}