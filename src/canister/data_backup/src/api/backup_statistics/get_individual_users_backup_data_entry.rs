use candid::Principal;
use shared_utils::{
    canister_specific::data_backup::types::all_user_data::AllUserData,
    common::types::{known_principal::KnownPrincipalType, storable_principal::StorablePrincipal},
};

use crate::CANISTER_DATA;

#[ic_cdk_macros::query]
#[candid::candid_method(query)]
fn get_individual_users_backup_data_entry(
    principal_id_of_user_whose_data_is_being_queried: Principal,
) -> Option<AllUserData> {
    let caller_principal_id = ic_cdk::caller();

    let global_super_admin_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .heap_data
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .cloned()
            .unwrap()
    });

    if caller_principal_id != global_super_admin_principal_id {
        return None;
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_all_user_data_map
            .get(&StorablePrincipal(
                principal_id_of_user_whose_data_is_being_queried,
            ))
    })
}
