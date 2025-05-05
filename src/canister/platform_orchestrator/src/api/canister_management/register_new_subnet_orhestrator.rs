use candid::Principal;
use ic_cdk::{
    api::management_canister::main::{canister_info, CanisterInfoRequest},
    id,
};
use ic_cdk_macros::update;
use ic_stable_structures::Storable;
use shared_utils::common::types::wasm::WasmType;

use crate::{guard::is_caller::is_caller_platform_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn register_new_subnet_orchestrator(
    new_subnet_orchestrator_caniter_id: Principal,
    subnet_is_available_for_provisioning_individual_canister: bool,
) -> Result<(), String> {
    let (new_subnet_orchestrator_canister_info,) = canister_info(CanisterInfoRequest {
        canister_id: new_subnet_orchestrator_caniter_id,
        num_requested_changes: None,
    })
    .await
    .map_err(|e| e.1)?;

    if !new_subnet_orchestrator_canister_info
        .controllers
        .contains(&id())
    {
        return Err(format!(
            "Controller of the new subnet orchestrator should be {}",
            id().to_text()
        ));
    }

    if let Some(first_subnet_orchestrator) = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .all_subnet_orchestrator_canisters_list
            .iter()
            .next()
            .copied()
    }) {
        let existing_subnet_orchestrator_canister_info_res = canister_info(CanisterInfoRequest {
            canister_id: first_subnet_orchestrator,
            num_requested_changes: None,
        })
        .await;

        if let Ok((existing_subnet_orchestrator_canister_info,)) =
            existing_subnet_orchestrator_canister_info_res
        {
            if !existing_subnet_orchestrator_canister_info
                .module_hash
                .eq(&new_subnet_orchestrator_canister_info.module_hash)
            {
                return Err(
                    format!("Canister Id has module hash {} which does not match exiting module hash of subnet orchestrator {}", String::from_utf8(new_subnet_orchestrator_canister_info.module_hash.unwrap_or_default()).unwrap_or_default(), String::from_utf8(existing_subnet_orchestrator_canister_info.module_hash.unwrap_or_default()).unwrap_or_default()),
                );
            }
        }
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .all_subnet_orchestrator_canisters_list
            .insert(new_subnet_orchestrator_caniter_id);

        if subnet_is_available_for_provisioning_individual_canister {
            canister_data
                .subet_orchestrator_with_capacity_left
                .insert(new_subnet_orchestrator_caniter_id);
        }
        Ok(())
    })
}
