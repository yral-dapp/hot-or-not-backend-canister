use ic_cdk::api::management_canister::{main, provisional::CanisterIdRecord};
use shared_utils::{
    common::types::known_principal::KnownPrincipalType, constant::RECHARGE_CYCLES_AMOUNT,
};

use crate::CANISTER_DATA;

#[ic_cdk_macros::update]
#[candid::candid_method(update)]
async fn return_cycles_to_user_index_canister() {
    let api_caller = ic_cdk::caller();

    let global_controller_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
            .cloned()
            .unwrap()
    });

    if api_caller != global_controller_principal_id {
        return;
    }

    let user_index_canister_principal_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .cloned()
            .unwrap()
    });

    main::deposit_cycles(
        CanisterIdRecord {
            canister_id: user_index_canister_principal_id,
        },
        RECHARGE_CYCLES_AMOUNT,
    )
    .await
    .unwrap();
}
