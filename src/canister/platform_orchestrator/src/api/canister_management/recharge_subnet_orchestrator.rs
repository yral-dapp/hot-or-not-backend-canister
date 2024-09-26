use ic_cdk::caller;
use ic_cdk_macros::update;

use crate::utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator;

#[update]
async fn recharge_subnet_orchestrator() -> Result<(), String> {
    let registered_subnet = RegisteredSubnetOrchestrator::new(caller())?;
    registered_subnet.deposit_cycles().await
}
