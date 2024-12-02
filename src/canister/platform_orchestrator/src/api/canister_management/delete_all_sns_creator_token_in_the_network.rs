use ic_cdk_macros::update;
use shared_utils::common::utils::task::run_task_concurrently;

use crate::{
    guard::is_caller::is_caller_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator, CANISTER_DATA,
};

#[update(guard = "is_caller_global_admin_or_controller")]
pub async fn delete_all_sns_creator_token_in_the_network() {
    let subnet_orchestrator_canister_ids = CANISTER_DATA
        .with_borrow(|canister_data| canister_data.subnet_orchestrators().clone());

    let delete_sns_creator_token_in_subnet_tasks = subnet_orchestrator_canister_ids
        .into_iter()
        .map(|subnet_orchestrator_canister_id| async move {
            let registered_subnet_orchestrator =
                RegisteredSubnetOrchestrator::new(subnet_orchestrator_canister_id)?;

            registered_subnet_orchestrator
                .delete_all_sns_creator_token_in_the_network()
                .await
        });

    run_task_concurrently(
        delete_sns_creator_token_in_subnet_tasks,
        10,
        |_| {},
        || false,
    )
    .await
}
