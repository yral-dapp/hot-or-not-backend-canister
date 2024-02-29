use ic_cdk::{api::{canister_balance, canister_balance128, is_controller, management_canister::{main, provisional::CanisterIdRecord}}, caller};
use ic_cdk_macros::update;
use shared_utils::{
    common::types::known_principal::KnownPrincipalType,
    constant::INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT,
};

use crate::CANISTER_DATA;

#[update]
async fn return_cycles_to_user_index_canister(cycle_amount: Option<u128>) {

    if !is_controller(&caller()) {
        panic!("Unauthorized")
    }


    let user_index_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdUserIndex)
            .cloned()
            .unwrap()
    });

    
    if cycle_amount.is_some() || canister_balance128() > INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT {
        main::deposit_cycles(
            CanisterIdRecord {
                canister_id: user_index_canister_id,
            },
            cycle_amount.unwrap_or(canister_balance128() - INDIVIDUAL_USER_CANISTER_RECHARGE_AMOUNT),
        )
        .await
        .unwrap();
    }
}
