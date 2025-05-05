use std::error::Error;

use candid::Principal;
use ic_cdk::api::management_canister::main::{canister_info, CanisterInfoRequest};
use ic_cdk_macros::update;
use shared_utils::common::types::wasm::WasmType;

use crate::{
    guard::is_caller::is_caller_platform_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator, CANISTER_DATA,
};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn upgrade_specific_individual_canister_with_version(
    individual_canister_id: Principal,
    version: String,
) -> Result<(), String> {
    upgrade_individual_canister_to_particular_version_impl(individual_canister_id, &version)
        .await
        .map_err(|e| e.to_string())
}

pub async fn upgrade_individual_canister_to_particular_version_impl(
    individual_canister_id: Principal,
    version: &str,
) -> Result<(), Box<dyn Error>> {
    let individual_canister_info = canister_info(CanisterInfoRequest {
        canister_id: individual_canister_id,
        num_requested_changes: None,
    })
    .await
    .map_err(|e| -> Box<dyn Error> { format!("{:?} {}", e.0, e.1).into() })?
    .0;

    let registered_subnet_orchestrator =
        RegisteredSubnetOrchestrator::new(individual_canister_info.controllers[0])?;

    let wasm = CANISTER_DATA.with_borrow(|canister_data| {
        let wasm_blob = canister_data
            .subnet_canister_upgrade_log
            .iter()
            .find(|canister_upgrade_status| {
                canister_upgrade_status.upgrade_arg.version.eq(version)
                    && canister_upgrade_status
                        .upgrade_arg
                        .canister
                        .eq(&WasmType::IndividualUserWasm)
            })
            .ok_or::<Box<dyn Error>>(
                format!("Canister Wasm for version {version} not found").into(),
            )?
            .upgrade_arg
            .wasm_blob;

        Ok::<_, Box<dyn Error>>(wasm_blob)
    })?;

    registered_subnet_orchestrator
        .upgrade_specific_individual_canister_with_wasm_version(
            individual_canister_id,
            version.to_string(),
            wasm,
        )
        .await?;

    Ok(())
}
