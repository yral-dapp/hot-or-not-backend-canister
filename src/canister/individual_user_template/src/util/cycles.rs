use candid::Principal;
use ic_cdk::call;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

use super::subnet_orchestrator::{self, SubnetOrchestrator};

pub async fn request_cycles_from_subnet_orchestrator(amount: u128) -> Result<(), String> {
    let subnet_orchestrator_canister_id = CANISTER_DATA
        .with_borrow(|canister_data| {
            canister_data
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdUserIndex)
                .copied()
        })
        .ok_or("Subnet Orchestrator Canister Id not found".to_owned())?;

    let result = call::<_, (Result<(), String>,)>(
        subnet_orchestrator_canister_id,
        "request_cycles",
        (amount,),
    )
    .await
    .map_err(|e| e.1)?
    .0;

    result
}

pub fn notify_to_recharge_canister() {
    if let Ok(subnet_orchestrator) = SubnetOrchestrator::new() {
        let res = subnet_orchestrator.notify_to_receive_cycles_from_subnet_orchestrator();
        if let Err(e) = res {
            ic_cdk::println!("Recharging canister failed. Error: {}", e)
        }
    } else {
        ic_cdk::println!("Recharging canister failed. Error: Subnet orchestrator id not found")
    }
}

pub async fn recharge_canister() {
    if let Ok(subnet_orchestrator) = SubnetOrchestrator::new() {
        let res = subnet_orchestrator
            .receive_cycles_from_subnet_orchestrator()
            .await;
        if let Err(e) = res {
            ic_cdk::println!("Recharging canister failed. Error: {}", e)
        }
    } else {
        ic_cdk::println!("Recharging canister failed. Error: Subnet orchestrator id not found")
    }
}
