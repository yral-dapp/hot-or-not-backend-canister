use std::collections::HashSet;

use candid::Principal;
use ic_cdk::{
    api::management_canister::main::{update_settings, CanisterSettings, UpdateSettingsArgument},
    notify,
};
use shared_utils::common::{
    types::known_principal::KnownPrincipalType, utils::task::run_task_concurrently,
};

use crate::{
    api::canister_management::set_controller_as_subnet_orchestrator::set_controller_as_subnet_orchestrator,
    CANISTER_DATA,
};

pub(crate) struct SubnetOrchestrator {
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

    pub async fn allot_empty_canister(&self) -> Result<Principal, String> {
        let canister_id: Principal = ic_cdk::call::<_, (Result<Principal, String>,)>(
            self.canister_id,
            "allot_empty_canister",
            (),
        )
        .await
        .map_err(|e| e.1)?
        .0?;

        Ok(canister_id)
    }

    pub fn notify_to_receive_cycles_from_subnet_orchestrator(&self) -> Result<(), String> {
        ic_cdk::notify(self.canister_id, "recharge_individual_user_canister", ())
            .map_err(|e| format!("notify to recharge individual user canister failed {:?}", e))
    }

    pub async fn receive_cycles_from_subnet_orchestrator(&self) -> Result<(), String> {
        ic_cdk::call::<_, (Result<(), String>,)>(
            self.canister_id,
            "recharge_individual_user_canister",
            (),
        )
        .await
        .map_err(|e| format!("recharge individual user canister failed {}", e.1))?
        .0
    }

    pub async fn insert_into_backup_pool(
        &self,
        canister_ids: Vec<Principal>,
    ) -> Result<(), Vec<Principal>> {
        let mut errored_canisters = vec![];
        let insert_canisters_into_subnet_backup_pool_task =
            canister_ids
                .clone()
                .into_iter()
                .map(|canister_id| async move {
                    update_settings(UpdateSettingsArgument {
                        canister_id,
                        settings: CanisterSettings {
                            controllers: Some(vec![self.canister_id]),
                            ..Default::default()
                        },
                    })
                    .await
                    .map_err(|e| (canister_id, e.1))
                });

        let result_callback = |result: Result<(), (Principal, String)>| match result {
            Ok(()) => {}
            Err(e) => {
                errored_canisters.push(e.0);
            }
        };

        run_task_concurrently(
            insert_canisters_into_subnet_backup_pool_task,
            10,
            result_callback,
            || false,
        )
        .await;

        let canister_ids_that_can_be_sent: Vec<Principal> = canister_ids
            .iter()
            .copied()
            .filter(|canister_id| !errored_canisters.contains(canister_id))
            .collect();

        ic_cdk::call::<_, (Result<(), String>,)>(
            self.canister_id,
            "receive_empty_canister_from_individual_canister",
            (canister_ids_that_can_be_sent.clone(),),
        )
        .await
        .inspect_err(|_| errored_canisters.extend(&canister_ids_that_can_be_sent))
        .map_err(|_e| errored_canisters.clone())?
        .0
        .inspect_err(|_| errored_canisters.extend_from_slice(&canister_ids_that_can_be_sent))
        .map_err(|_| (errored_canisters.clone()))?;

        if errored_canisters.is_empty() {
            Ok(())
        } else {
            Err(errored_canisters)
        }
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
