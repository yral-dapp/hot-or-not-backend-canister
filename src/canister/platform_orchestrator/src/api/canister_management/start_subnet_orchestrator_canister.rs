use crate::{
    guard::is_caller::is_caller_platform_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator,
};
use candid::Principal;
use ic_cdk_macros::update;

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn start_subnet_orchestrator_canister(
    subnet_orchestrator_canister_id: Principal,
) -> Result<(), String> {
    let subnet_orchestrator = RegisteredSubnetOrchestrator::new(subnet_orchestrator_canister_id)?;
    subnet_orchestrator.start_canister().await
}
