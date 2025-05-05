use candid::Principal;
use ic_cdk::api::management_canister::main::{canister_info, CanisterInfoRequest};
use ic_cdk_macros::update;

use crate::{
    guard::is_caller::is_caller_platform_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator,
};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
pub async fn notify_specific_individual_canister_to_upgrade_creator_dao_governance_canisters(
    individual_canister_id: Principal,
    wasm_module: Vec<u8>,
) -> Result<(), String> {
    let (individual_canister_info,) = canister_info(CanisterInfoRequest {
        canister_id: individual_canister_id,
        num_requested_changes: None,
    })
    .await
    .map_err(|e| e.1)?;

    let subnet_orchestrator_canister_id = individual_canister_info
        .controllers
        .get(0)
        .copied()
        .ok_or("Subnet orchestrator canister id not found")?;

    let registered_subnet_orchestrator =
        RegisteredSubnetOrchestrator::new(subnet_orchestrator_canister_id)?;

    registered_subnet_orchestrator
        .notify_specific_individual_canister_to_upgrade_creator_dao_governance_canisters(
            individual_canister_id,
            wasm_module,
        )
}
