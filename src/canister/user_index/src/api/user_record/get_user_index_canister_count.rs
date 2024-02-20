use ic_cdk_macros::query;

use crate::CANISTER_DATA;

#[query]
fn get_user_index_canister_count() -> usize {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_canister_id_map
            .len()
    })
}
