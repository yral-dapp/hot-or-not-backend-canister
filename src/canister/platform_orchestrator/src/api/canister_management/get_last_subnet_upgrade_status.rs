use candid::candid_method;

use crate::{data_model::CanisterUpgradeStatus, CANISTER_DATA};



#[candid_method(query)]
#[ic_cdk::query]
pub fn get_subnet_last_upgrade_status() -> CanisterUpgradeStatus {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.last_subnet_canister_upgrade_status.clone()
    })
}