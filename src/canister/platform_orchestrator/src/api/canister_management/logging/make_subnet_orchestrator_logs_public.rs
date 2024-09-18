use candid::Principal;

use crate::{
    guard::is_caller::is_caller_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator,
};

#[ic_cdk_macros::update(guard = "is_caller_global_admin_or_controller")]
async fn make_subnet_orchestrator_logs_public(canister_id: Principal) -> Result<(), String> {
    let registered_subnet_orchestrator = RegisteredSubnetOrchestrator::new(canister_id)?;
    registered_subnet_orchestrator.make_logs_public().await
}
