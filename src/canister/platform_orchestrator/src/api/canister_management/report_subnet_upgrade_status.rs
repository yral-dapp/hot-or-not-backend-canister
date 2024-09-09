use ic_cdk::caller;
use shared_utils::canister_specific::user_index::types::UpgradeStatus;

use crate::{utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator, CANISTER_DATA};

#[ic_cdk_macros::update]
pub fn report_subnet_upgrade_status(subnet_upgrade_status: UpgradeStatus) -> Result<(), String> {
    let registered_subnet_orchestrator = RegisteredSubnetOrchestrator::new(caller())?;
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .subnets_upgrade_report
            .subnet_wise_report
            .insert(
                registered_subnet_orchestrator.get_canister_id(),
                subnet_upgrade_status,
            )
    });
    Ok(())
}
