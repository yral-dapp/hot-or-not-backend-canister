use candid::Principal;
use ic_cdk_macros::update;

use crate::{
    guard::is_caller::is_caller_platform_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator,
};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn set_reserved_cycle_limit_for_subnet_orchestrator(
    subnet_orchestrator_canister_id: Principal,
    amount: u128,
) -> Result<(), String> {
    let registered_subnet_orchestrator =
        RegisteredSubnetOrchestrator::new(subnet_orchestrator_canister_id)?;
    registered_subnet_orchestrator
        .set_reserved_cycle_limit(amount)
        .await
}
