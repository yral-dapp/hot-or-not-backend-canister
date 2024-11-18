use std::collections::HashSet;

use candid::Principal;
use ic_cdk::notify;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::CANISTER_DATA;

pub struct SubnetOrchestrator {
    canister_id: Principal,
}

impl SubnetOrchestrator {
    pub fn new() -> Result<Self, String> {
        let subnet_orchestrator = CANISTER_DATA.with_borrow(|canister_data| {
            let canister_id = canister_data
                .known_principal_ids
                .get(&KnownPrincipalType::CanisterIdUserIndex)
                .copied();

            canister_id.map(|canister_id| Self { canister_id })
        });

        subnet_orchestrator.ok_or("Subnet Orchestrator canister not found".into())
    }

    pub async fn get_empty_canister(&self) -> Result<Principal, String> {
        let (result,) = ic_cdk::call(self.canister_id, "allot_empty_canister", ())
            .await
            .map_err(|e| e.1)?;

        result
    }

    pub fn receive_cycles_from_subnet_orchestrator(&self) -> Result<(), String> {
        ic_cdk::notify(self.canister_id, "recharge_individual_user_canister", ())
            .map_err(|e| format!("notify to recharge individual user canister failed {:?}", e))
    }

    pub fn send_creator_dao_stats(&self, root_canisters: HashSet<Principal>) -> Result<(), String> {
        notify(
            self.canister_id,
            "receive_creator_dao_stats_from_individual_canister",
            (root_canisters,),
        )
        .map_err(|e| {
            format!(
                "error sending canister stats to subnet orchestrator {:?}",
                e
            )
        })
    }
}
