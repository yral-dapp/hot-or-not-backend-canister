use shared_utils::canister_specific::platform_orchestrator::types::SubnetUpgradeReport;

use crate::CANISTER_DATA;

#[ic_cdk_macros::query]
pub async fn get_subnets_upgrade_status_report() -> SubnetUpgradeReport {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.subnets_upgrade_report.clone())
}
