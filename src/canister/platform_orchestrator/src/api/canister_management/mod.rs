use ic_cdk_macros::query;

use crate::CANISTER_DATA;


mod provision_subnet_orchestrator;
mod upgrade_canister;
mod upload_wasms;
mod subnet_orchestrator_maxed_out;
mod get_last_subnet_upgrade_status;
mod get_all_available_subnet_orchestrators;
mod get_all_subnet_orchestrators;

#[query]
pub fn get_version() -> String {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.version_detail.version.clone()
    })
}