use ic_cdk::api::call::{self, CallResult};
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

#[ic_cdk::update]
#[candid::candid_method(update)]
async fn backup_all_individual_user_canisters() {
    let api_caller = ic_cdk::caller();

    let global_super_admin_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .cloned()
            .unwrap()
    });

    if api_caller != global_super_admin_principal_id {
        return;
    }

    let all_individual_user_canister_ids = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_canister_id_map
            .clone()
    });

    for user_canister_id in all_individual_user_canister_ids.values() {
        let upgrade_response: CallResult<()> = call::call(
            user_canister_id.clone(),
            "backup_data_to_backup_canister",
            (),
        )
        .await;
        upgrade_response.ok();
    }
}
