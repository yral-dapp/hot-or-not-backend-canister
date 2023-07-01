use crate::CANISTER_DATA;

#[ic_cdk::update]
#[candid::candid_method(update)]
fn get_user_index_canister_count() -> usize {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_canister_id_map
            .len()
    })
}
