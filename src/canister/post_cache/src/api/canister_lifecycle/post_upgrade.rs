use ic_cdk::storage;

use crate::{
    data_model::{CanisterData, CanisterDataV2},
    CANISTER_DATA_V2,
};

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    match storage::stable_restore::<(CanisterDataV2,)>() {
        Ok((canister_data,)) => {
            CANISTER_DATA_V2.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(_) => match storage::stable_restore::<(CanisterData,)>() {
            Ok((canister_data,)) => {
                let canister_data_v2 = canister_data.into();
                CANISTER_DATA_V2.with(|canister_data_ref_cell| {
                    *canister_data_ref_cell.borrow_mut() = canister_data_v2;
                });
            }
            Err(_) => panic!("Failed to decode with both V1 and V2"),
        },
    }
}
