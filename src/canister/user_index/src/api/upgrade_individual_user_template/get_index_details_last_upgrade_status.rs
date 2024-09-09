use ic_cdk_macros::query;

use crate::CANISTER_DATA;
use shared_utils::canister_specific::user_index::types::UpgradeStatus;

#[query]
fn get_index_details_last_upgrade_status() -> UpgradeStatus {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .last_run_upgrade_status
            .clone()
    })
}
