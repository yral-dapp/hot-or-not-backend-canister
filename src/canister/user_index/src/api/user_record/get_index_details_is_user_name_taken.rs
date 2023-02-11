use crate::CANISTER_DATA;

#[ic_cdk_macros::query]
#[candid::candid_method(query)]
fn get_index_details_is_user_name_taken(user_name: String) -> bool {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .unique_user_name_to_user_principal_id_map
            .contains_key(&user_name)
    })
}
