use candid::Principal;
use ic_cdk::api::management_canister::main::{uninstall_code, CanisterIdRecord};
use shared_utils::common::utils::task::run_task_concurrently;

use crate::CANISTER_DATA;

use super::SubnetOrchestrator;

pub(crate) async fn uninstall_code_and_return_empty_canisters_to_subnet_backup_pool(
    canister_ids: Vec<Principal>,
) {
    let mut failed_to_return_canister_ids = vec![];
    let subnet_orchestrator_res = SubnetOrchestrator::new().inspect_err(|_e| {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data
                .empty_canisters
                .append_empty_canisters(canister_ids.clone())
        })
    });

    if let Ok(subnet_orchestrator) = subnet_orchestrator_res {
        let uninstall_code_tasks = canister_ids.iter().map(|canister_id| async {
            uninstall_code(CanisterIdRecord {
                canister_id: *canister_id,
            })
            .await
            .map_err(|e| (*canister_id, format!("{:?}", e)))
        });

        let uninstall_callback = |uninstall_result: Result<(), (Principal, String)>| {
            if let Err(e) = uninstall_result {
                ic_cdk::println!("Error Uninstall Code from canister {}. Error: {}", e.0, e.1);
                failed_to_return_canister_ids.push(e.0);
            }
        };

        run_task_concurrently(uninstall_code_tasks, 10, uninstall_callback, || false).await;

        let inserting_canisters_into_subnet_backup_pool_res = subnet_orchestrator
            .insert_into_backup_pool(
                canister_ids
                    .into_iter()
                    .filter(|canister_id| !failed_to_return_canister_ids.contains(canister_id))
                    .collect(),
            )
            .await;

        if let Err(e) = inserting_canisters_into_subnet_backup_pool_res {
            failed_to_return_canister_ids.extend(e.into_iter());
        }

        CANISTER_DATA.with_borrow_mut(|canister_data| {
            canister_data
                .empty_canisters
                .append_empty_canisters(failed_to_return_canister_ids);
        });
    }
}
