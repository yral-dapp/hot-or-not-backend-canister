use ic_cdk_macros::update;
use shared_utils::common::{types::wasm, utils::task::run_task_concurrently};

use crate::{guard::is_caller::is_caller_platform_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
pub fn upgrade_all_creator_dao_governance_canisters_in_the_network(wasm_module: Vec<u8>) {
    let subnet_orchestrators = CANISTER_DATA
        .with_borrow(|canister_data| canister_data.all_subnet_orchestrator_canisters_list.clone());

    let upgrade_governance_canister_tasks =
        subnet_orchestrators
            .into_iter()
            .map(move |subnet_orchestrator| {
                let wasm = wasm_module.clone();
                async move {
                    ic_cdk::call::<_, ()>(
                        subnet_orchestrator,
                        "upgrade_all_creator_dao_governance_canisters_in_the_network",
                        (wasm,),
                    )
                    .await
                    .map_err(|e| e.1)
                }
            });

    ic_cdk::spawn(run_task_concurrently(
        upgrade_governance_canister_tasks,
        10,
        |result| {
            if let Err(e) = result {
                ic_cdk::println!("Error upgrading governance canister in the subnet. {}", e);
            }
        },
        || false,
    ));
}
