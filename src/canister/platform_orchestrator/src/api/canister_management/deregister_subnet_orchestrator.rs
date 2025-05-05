use candid::Principal;
use ic_cdk_macros::update;

use crate::{guard::is_caller::is_caller_platform_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
fn deregister_subnet_orchestrator(canister_id: Principal, remove_it_completely: bool) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .subet_orchestrator_with_capacity_left
            .remove(&canister_id);

        if remove_it_completely {
            canister_data
                .all_subnet_orchestrator_canisters_list
                .remove(&canister_id);
        }
    });
}
