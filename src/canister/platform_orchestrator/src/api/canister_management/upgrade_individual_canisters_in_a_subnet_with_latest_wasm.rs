use candid::Principal;
use ic_cdk_macros::update;

use crate::{
    guard::is_caller::is_caller_platform_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator,
};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn upgrade_individual_canisters_in_a_subnet_with_latest_wasm(
    subnet_orchestrator_canister_id: Principal,
) -> Result<(), String> {
    let registered_subnet_orchestrator =
        RegisteredSubnetOrchestrator::new(subnet_orchestrator_canister_id)?;

    registered_subnet_orchestrator
        .upgrade_individual_canisters_in_subnet_with_latest_wasm()
        .await
}
