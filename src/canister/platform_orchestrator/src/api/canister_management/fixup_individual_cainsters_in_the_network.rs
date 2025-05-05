use ic_cdk_macros::update;
use shared_utils::common::utils::task::run_task_concurrently;

use crate::{
    guard::is_caller::is_caller_platform_global_admin_or_controller,
    utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator, CANISTER_DATA,
};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn fixup_individual_cainsters_in_thebreaking_condition_network() {
    let subnet_orchestrators = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .all_subnet_orchestrator_canisters_list
            .clone()
            .into_iter()
    });
    let fixup_individual_canisters_in_subnet_futures =
        subnet_orchestrators.map(|subnet_orchestrator| async move {
            let registered_subnetorchestrator =
                RegisteredSubnetOrchestrator::new(subnet_orchestrator)?;
            registered_subnetorchestrator
                .fixup_individual_cansiters_mapping()
                .await
        });

    run_task_concurrently(
        fixup_individual_canisters_in_subnet_futures,
        10,
        |_| {},
        || false,
    )
    .await;
}
