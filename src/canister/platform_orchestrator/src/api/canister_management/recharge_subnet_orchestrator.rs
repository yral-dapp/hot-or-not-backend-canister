use ic_cdk::{
    api::management_canister::main::{deposit_cycles, CanisterIdRecord},
    caller,
};
use ic_cdk_macros::update;
use shared_utils::constant::SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES;

use crate::{utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator, CANISTER_DATA};

#[update]
async fn recharge_subnet_orchestrator() -> Result<(), String> {
    let registered_subnet = RegisteredSubnetOrchestrator::new(caller())?;
    registered_subnet
        .deposit_cycles(SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES)
        .await
}
