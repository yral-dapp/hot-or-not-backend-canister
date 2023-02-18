use ic_cdk::storage;

use crate::CANISTER_DATA;

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok((canister_data,)) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(_) => {
            panic!("Failed to restore canister data from stable memory");
        }
    }
}
