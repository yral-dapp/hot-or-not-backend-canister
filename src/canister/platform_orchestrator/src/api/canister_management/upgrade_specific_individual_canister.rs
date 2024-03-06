use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller;

use crate::CANISTER_DATA; 


#[update(guard = "is_caller_controller")]
fn upgrade_specific_individual_canister(individual_canister_id: Principal) {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.all_subnet_orchestrator_canisters_list.iter().for_each(|subnet_id| {
            let _ = ic_cdk::notify(*subnet_id, "upgrade_specific_individual_user_canister_with_latest_wasm", (individual_canister_id, ));
        })
    })
}   