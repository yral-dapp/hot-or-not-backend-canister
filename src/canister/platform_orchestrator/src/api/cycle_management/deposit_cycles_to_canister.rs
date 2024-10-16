use candid::Principal;
use ic_cdk::{
    api::management_canister::main::{deposit_cycles, CanisterIdRecord},
    update,
};

use crate::guard::is_caller::is_caller_global_admin_or_controller;

#[update(guard = "is_caller_global_admin_or_controller")]
async fn deposit_cycles_to_canister(
    canister_id: Principal,
    cycles: u128,
) -> Result<String, String> {
    deposit_cycles(CanisterIdRecord { canister_id }, cycles)
        .await
        .map_err(|e| e.1)?;

    Ok(String::from("Success"))
}
