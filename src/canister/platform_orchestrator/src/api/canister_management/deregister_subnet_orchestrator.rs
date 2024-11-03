use candid::Principal;
use ic_cdk_macros::update;

use crate::{guard::is_caller::is_caller_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_global_admin_or_controller")]
fn deregister_subnet_orchestrator(canister_id: Principal, remove_it_completely: bool) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.remove_subnet_orchestrator(canister_id, remove_it_completely);
    });
}
