use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

#[ic_cdk::update]
#[candid::candid_method(update)]
fn send_restore_data_back_to_user_index_canister() {
    // * Get the caller principal ID.
    let caller_principal_id = ic_cdk::caller();

    if !(CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .heap_data
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .unwrap()
            .clone()
            == caller_principal_id
    })) {
        return;
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let user_index_canister_id = canister_data_ref_cell
            .borrow()
            .heap_data
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .unwrap()
            .clone();

        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_all_user_data_map
            .iter()
            .for_each(
                |(storable_user_principal_id, corresponding_all_user_data)| {
                    ic_cdk::notify(
                        user_index_canister_id,
                        "receive_data_from_backup_canister_and_restore_data_to_heap",
                        (
                            storable_user_principal_id.0,
                            corresponding_all_user_data.user_canister_id,
                            corresponding_all_user_data.canister_data.unique_user_name,
                        ),
                    )
                    .unwrap_or_default();
                },
            );
    });
}
