use ic_cdk::{
    api::management_canister::main::{deposit_cycles, CanisterIdRecord},
    caller,
};
use ic_cdk_macros::update;
use shared_utils::constant::SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES;

use crate::CANISTER_DATA;

#[update]
async fn recharge_subnet_orchestrator() -> Result<(), String> {
    let subnet_orchestrator_canister_id = caller();
    let contains_caller_subnet = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .all_subnet_orchestrator_canisters_list
            .contains(&subnet_orchestrator_canister_id)
    });

    if !contains_caller_subnet {
        return Err("Subnet Orchestrator not found".into());
    }

    deposit_cycles(
        CanisterIdRecord {
            canister_id: subnet_orchestrator_canister_id,
        },
        SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES,
    )
    .await
    .map_err(|err| err.1)
}
