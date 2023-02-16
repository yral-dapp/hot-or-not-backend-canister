use ic_cdk::storage;

use crate::CANISTER_DATA;

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok((canister_data,)) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(e) => {
            ic_cdk::print(format!("{}", e));
            panic!("Failed to restore canister data from stable memory");
        }
    }
}
