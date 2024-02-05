use candid::candid_method;

use crate::CANISTER_DATA;


mod provision_subnet_orchestrator;
mod upgrade_canister;
mod upload_wasms;
mod subnet_orchestrator_maxed_out;

#[ic_cdk::query]
#[candid_method(query)]
pub fn get_version() -> String {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.version_detail.version.clone()
    })
}