use candid::Principal;
use ic_cdk::api::management_canister::main::{canister_info, CanisterInfoRequest};
use ic_cdk_macros::update;

use crate::{
    guard::is_caller::is_caller_platform_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator,
};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
pub async fn delete_all_sns_creator_token_of_an_individual_canister(
    individual_canister_id: Principal,
) -> Result<(), String> {
    let (canister_info,) = canister_info(CanisterInfoRequest {
        canister_id: individual_canister_id,
        num_requested_changes: None,
    })
    .await
    .map_err(|e| e.1)?;

    let subnet_orchestrator_canister_id = canister_info.controllers[0];
    let subnet_orchestrator = RegisteredSubnetOrchestrator::new(subnet_orchestrator_canister_id)?;

    subnet_orchestrator
        .delete_all_sns_creator_token_for_an_individual_canister(individual_canister_id)
        .await
}
