use candid::Principal;
use ic_cdk::api::management_canister::main::{deposit_cycles, CanisterId, CanisterIdRecord};

use crate::CANISTER_DATA;

pub struct RegisteredSubnetOrchestrator {
    canister_id: Principal,
}

impl RegisteredSubnetOrchestrator {
    pub fn new(canister_id: Principal) -> Result<RegisteredSubnetOrchestrator, String> {
        let contains = CANISTER_DATA.with_borrow(|canister_data| {
            canister_data
                .all_subnet_orchestrator_canisters_list
                .contains(&canister_id)
        });

        if contains {
            Ok(RegisteredSubnetOrchestrator { canister_id })
        } else {
            Err("Canister Id is not found in platform orchestrator".into())
        }
    }

    pub fn get_canister_id(&self) -> Principal {
        self.canister_id
    }

    pub async fn deposit_cycles(&self, cycles: u128) -> Result<(), String> {
        deposit_cycles(
            CanisterIdRecord {
                canister_id: self.canister_id,
            },
            cycles,
        )
        .await
        .map_err(|e| e.1)
    }
}
