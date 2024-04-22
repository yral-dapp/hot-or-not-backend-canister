use ic_cdk_macros::query;

use crate::{data_model::CanisterData, CANISTER_DATA};

#[query]
fn reset_data() {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        *canister_data = CanisterData::default();
    });
}
