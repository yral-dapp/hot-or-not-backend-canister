use ic_cdk::{
    api::management_canister::main::{deposit_cycles, CanisterIdRecord},
    caller,
};
use ic_cdk_macros::update;

use crate::{
    util::canister_management::check_and_request_cycles_from_platform_orchestrator, CANISTER_DATA,
};

#[update]
async fn request_cycles(cycle_amount: u128) -> Result<(), String> {
    let canister_id = caller();

    let found_canister_id = CANISTER_DATA.with_borrow(|canister_data| {
        let res = canister_data
            .user_principal_id_to_canister_id_map
            .iter()
            .find(|(_, id)| **id == canister_id);

        res.is_some()
    });

    if !found_canister_id {
        return Err("Unauthorized".into());
    }

    let recharge_amount = u128::min(cycle_amount, 5_000_000_000_000);

    let _ = check_and_request_cycles_from_platform_orchestrator().await;

    deposit_cycles(CanisterIdRecord { canister_id }, recharge_amount)
        .await
        .map_err(|e| e.1)
}
