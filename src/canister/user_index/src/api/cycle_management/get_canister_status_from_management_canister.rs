use candid::Principal;
use ic_cdk::api::management_canister::{
    main::{self, CanisterStatusResponse},
    provisional::CanisterIdRecord,
};
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

// TODO: move this to the individual canisters
// TODO: Do this by calling this via the user_index canister
#[ic_cdk::update]
#[candid::candid_method(update)]
async fn get_canister_status_from_management_canister(
    canister_id: Principal,
) -> Result<CanisterStatusResponse, String> {
    let api_caller = ic_cdk::caller();

    let global_super_admin = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .unwrap()
            .clone()
    });

    if api_caller != global_super_admin {
        return Err(format!(
            "Only the global super admin can call this method. Caller: {:?}",
            api_caller
        ));
    }

    let (response,) = main::canister_status(CanisterIdRecord { canister_id })
        .await
        .map_err(|e| format!("Error calling canister_status: {:?}", e))?;

    Ok(response)
}
