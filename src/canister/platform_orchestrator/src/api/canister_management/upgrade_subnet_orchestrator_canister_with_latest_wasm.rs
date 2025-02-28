use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::platform_orchestrator::types::args::UpgradeCanisterArg,
    common::types::wasm::WasmType,
};

use crate::{
    guard::is_caller::is_caller_global_admin_or_controller,
    utils::{
        recharge_and_upgrade_subnet_orchestrator,
        registered_subnet_orchestrator::RegisteredSubnetOrchestrator,
        upgrade_subnet_orchestrator_canister,
    },
    CANISTER_DATA,
};

#[update(guard = "is_caller_global_admin_or_controller")]
pub async fn upgrade_subnet_orchestrator_canister_with_latest_wasm(
    subnet_orchestrator_cansiter_id: Principal,
) -> Result<(), String> {
    let registered_subnet_orhcestrator =
        RegisteredSubnetOrchestrator::new(subnet_orchestrator_cansiter_id)?;

    let subnet_orchestrator_wasm = CANISTER_DATA
        .with_borrow(|canister_data| canister_data.wasms.get(&WasmType::SubnetOrchestratorWasm));

    let subnet_orchestrator_wasm =
        subnet_orchestrator_wasm.ok_or("subnet orchestrator wasm not found".to_owned())?;

    recharge_and_upgrade_subnet_orchestrator(
        registered_subnet_orhcestrator.get_canister_id(),
        UpgradeCanisterArg {
            canister: WasmType::SubnetOrchestratorWasm,
            version: subnet_orchestrator_wasm.version,
            wasm_blob: subnet_orchestrator_wasm.wasm_blob,
        },
    )
    .await
    .map_err(|e| e.1)?;

    Ok(())
}
