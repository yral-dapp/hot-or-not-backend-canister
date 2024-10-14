use candid::Principal;
use ic_cdk_macros::update;

use crate::{
    guard::is_caller::is_caller_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator,
};

#[update(guard = "is_caller_global_admin_or_controller")]
async fn provision_empty_canisters_in_a_subnet(
    subnet_orchestrator_canister_id: Principal,
    number_of_canisters: u64,
) -> Result<(), String> {
    let registered_subnet_orchestrator =
        RegisteredSubnetOrchestrator::new(subnet_orchestrator_canister_id)?;
    registered_subnet_orchestrator
        .provision_empty_canisters(number_of_canisters)
        .await
}
