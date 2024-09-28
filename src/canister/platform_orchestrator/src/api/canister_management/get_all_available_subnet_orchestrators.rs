use candid::Principal;
use ic_cdk_macros::query;

use crate::CANISTER_DATA;

#[query]
fn get_all_available_subnet_orchestrators() -> Vec<Principal> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let canisters = canister_data
            .subet_orchestrator_with_capacity_left
            .iter()
            .copied()
            .collect::<Vec<Principal>>();
        canisters
    })
}
