use candid::Principal;
use ic_cdk::api::management_canister::main::{canister_info, CanisterInfoRequest};
use ic_cdk_macros::update;

use crate::{
    guard::is_caller::is_caller_platform_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator,
};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn upgrade_specific_individual_canister_with_wasm(
    individual_canister_id: Principal,
    version: String,
    individual_user_wasm: Vec<u8>,
) -> Result<(), String> {
    let individual_canister_info = canister_info(CanisterInfoRequest {
        canister_id: individual_canister_id,
        num_requested_changes: None,
    })
    .await
    .map_err(|e| format!("{:?} {}", e.0, e.1))?
    .0;

    let registered_subnet_orchestrator =
        RegisteredSubnetOrchestrator::new(individual_canister_info.controllers[0])?;

    registered_subnet_orchestrator
        .upgrade_specific_individual_canister_with_wasm_version(
            individual_canister_id,
            version,
            individual_user_wasm,
        )
        .await
}
