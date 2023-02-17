use ic_cdk::storage;

use crate::CANISTER_DATA_V2;

#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade() {
    // CANISTER_DATA.with(|canister_data_ref_cell| {
    //     let canister_data = canister_data_ref_cell.take();

    //     storage::stable_save((canister_data,)).ok();
    // });
    CANISTER_DATA_V2.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.take();

        storage::stable_save((canister_data,)).ok();
    });
}
