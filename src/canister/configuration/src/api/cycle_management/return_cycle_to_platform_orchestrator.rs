use candid::Principal;
use ic_cdk::api::{
    canister_balance128,
    management_canister::{main, provisional::CanisterIdRecord},
};
use std::str::FromStr;

#[ic_cdk::update]
async fn return_cycles_to_platform_orchestrator_canister() {
    let platform_orchestrator_canister_id = Principal::from_str("74zq4-iqaaa-aaaam-ab53a-cai").unwrap();

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
