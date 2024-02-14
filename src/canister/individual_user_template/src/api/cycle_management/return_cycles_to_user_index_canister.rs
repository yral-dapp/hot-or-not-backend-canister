use ic_cdk::api::management_canister::{main, provisional::CanisterIdRecord};
use ic_cdk_macros::update;
use shared_utils::{
    common::types::known_principal::KnownPrincipalType,
    constant::INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT,
};

use crate::CANISTER_DATA;

#[update]
async fn return_cycles_to_user_index_canister(cycle_amount: Option<u128>) {
    let api_caller = ic_cdk::caller();

    let user_index_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .cloned()
            .unwrap()
    });

    if api_caller != user_index_canister_id {
        return;
    }

    main::deposit_cycles(
        CanisterIdRecord {
            canister_id: user_index_canister_id,
        },
        cycle_amount.unwrap_or(INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT / 2),
    )
    .await
    .unwrap();
}
