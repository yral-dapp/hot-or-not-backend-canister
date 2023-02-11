use ic_cdk::api::management_canister::{main, provisional::CanisterIdRecord};
use shared_utils::{
    common::types::known_principal::KnownPrincipalType,
    constant::{
        CYCLES_THRESHOLD_TO_INITIATE_RECHARGE, MINIMUM_CYCLES_TO_REVIVE_CANISTER,
        RECHARGE_CYCLES_AMOUNT,
    },
};

use crate::CANISTER_DATA;

// TODO: Convert this to a daily cron job that is then moved to individual canisters
#[ic_cdk_macros::update]
#[candid::candid_method(update)]
async fn topup_canisters_that_need_it() {
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

    let user_principal_id_to_canister_id_map = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .user_principal_id_to_canister_id_map
            .clone()
    });

    for (_user_principal_id, user_canister_id) in user_principal_id_to_canister_id_map.iter() {
        let response_result = main::canister_status(CanisterIdRecord {
            canister_id: user_canister_id.clone(),
        })
        .await;

        if response_result.is_err() {
            main::deposit_cycles(
                CanisterIdRecord {
                    canister_id: user_canister_id.clone(),
                },
                MINIMUM_CYCLES_TO_REVIVE_CANISTER,
            )
            .await
            .unwrap();
        }

        let (response,) = main::canister_status(CanisterIdRecord {
            canister_id: user_canister_id.clone(),
        })
        .await
        .unwrap();

        if response.cycles < CYCLES_THRESHOLD_TO_INITIATE_RECHARGE {
            main::deposit_cycles(
                CanisterIdRecord {
                    canister_id: user_canister_id.clone(),
                },
                RECHARGE_CYCLES_AMOUNT,
            )
            .await
            .unwrap();
        }
    }
}
