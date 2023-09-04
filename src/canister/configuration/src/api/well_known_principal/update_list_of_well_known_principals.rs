use candid::Principal;
use ic_cdk::api;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::{data::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_list_of_well_known_principals(
    principal_type: KnownPrincipalType,
    principal_value: Principal,
) -> Result<(), String> {
    let api_caller = ic_cdk::caller();
    let super_admin = CANISTER_DATA
        .with(|canister_data_ref_cell| {
            canister_data_ref_cell
                .borrow()
                .known_principal_ids
                .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
                .cloned()
        })
        .ok_or("Super admin not found in internal records")?;
    // let user_index_canister_id = CANISTER_DATA
    //     .with(|canister_data_ref_cell| {
    //         canister_data_ref_cell
    //             .borrow()
    //             .known_principal_ids
    //             .get(&KnownPrincipalType::CanisterIdUserIndex)
    //             .cloned()
    //     })
    //     .ok_or("User index canister not found in internal records")?;
    // let post_cache_canister_id = CANISTER_DATA
    //     .with(|canister_data_ref_cell| {
    //         canister_data_ref_cell
    //             .borrow()
    //             .known_principal_ids
    //             .get(&KnownPrincipalType::CanisterIdPostCache)
    //             .cloned()
    //     })
    //     .ok_or("Post cache canister not found in internal records")?;

    validate_authorization(&super_admin, &api_caller)?;

    CANISTER_DATA.with(|canister_data_ref_cell| {
        upsert_value_into_list_of_known_principals(
            &mut canister_data_ref_cell.borrow_mut(),
            &principal_type,
            &principal_value,
        )
    });

    // TODO: enable these calls once the canisters are ready
    // update_canister_with_known_principal(
    //     &user_index_canister_id,
    //     &principal_type,
    //     &principal_value,
    // )?;

    // update_canister_with_known_principal(
    //     &post_cache_canister_id,
    //     &principal_type,
    //     &principal_value,
    // )?;

    Ok(())
}

fn validate_authorization(super_admin: &Principal, api_caller: &Principal) -> Result<(), String> {
    let is_super_admin = api_caller == super_admin;

    let is_canister_controller = api::is_controller(api_caller);

    if !is_super_admin && !is_canister_controller {
        return Err("Unauthorized".to_string());
    }

    Ok(())
}

fn upsert_value_into_list_of_known_principals(
    canister_data: &mut CanisterData,
    principal_type: &KnownPrincipalType,
    principal_value: &Principal,
) {
    canister_data
        .known_principal_ids
        .insert(*principal_type, *principal_value);
}

// fn update_canister_with_known_principal(
//     canister_id_of_canister_to_update: &Principal,
//     principal_type: &KnownPrincipalType,
//     principal_value: &Principal,
// ) -> Result<(), String> {
//     ic_cdk::api::call::notify(
//         *canister_id_of_canister_to_update,
//         "update_known_principals",
//         (principal_type, principal_value),
//     )
//     .map_err(|_| {
//         format!(
//             "Updating known principal failed for canister {}",
//             canister_id_of_canister_to_update.to_text()
//         )
//     })
// }
