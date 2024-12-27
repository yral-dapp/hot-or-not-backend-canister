use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::{
    permissions::is_caller_controller_or_global_admin, task::run_task_concurrently,
};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
pub fn upgrade_all_creator_dao_governance_canisters_in_the_network(wasm_module: Vec<u8>) {
    let individual_canisters: Vec<Principal> = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .user_principal_id_to_canister_id_map
            .values()
            .cloned()
            .collect()
    });

    let upgrade_governance_canister_tasks =
        individual_canisters.into_iter().map(move |canister_id| {
            let wasm = wasm_module.clone();
            async move {
                ic_cdk::call::<_, (Result<(), String>,)>(
                    canister_id,
                    "upgrade_creator_dao_governance_canisters",
                    (wasm,),
                )
                .await
                .map_err(|e| format!("Error: {:?}", e))?
                .0
            }
        });

    ic_cdk::spawn(run_task_concurrently(
        upgrade_governance_canister_tasks,
        10,
        |result| {
            if let Err(e) = result {
                ic_cdk::println!("Error upgrading governance canister. Error: {}", e);
            }
        },
        || false,
    ));
}
