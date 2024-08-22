use ic_cdk::api::{
    canister_balance128,
    management_canister::{main, provisional::CanisterIdRecord},
};
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

#[ic_cdk::update]
async fn return_cycles_to_platform_orchestrator_canister() {
    let platform_orchestrator_canister_id = CANISTER_DATA.with(|canister_data_ref_cell| {
        canister_data_ref_cell
            .borrow()
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
            .cloned()
            .unwrap()
    });

    if canister_balance128() > 200_000_000_000 {
        let cycle_amount_to_transfer = canister_balance128() - 200_000_000_000;
        main::deposit_cycles(
            CanisterIdRecord {
                canister_id: platform_orchestrator_canister_id,
            },
            cycle_amount_to_transfer,
        )
        .await
        .unwrap();
    }
}
