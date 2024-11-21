use candid::Principal;
use ic_cdk::call;
use ic_cdk_macros::update;
use shared_utils::common::utils::{permissions::is_caller_controller, task::run_task_concurrently};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller")]
fn reclaim_cycles_from_individual_canisters() {
    ic_cdk::spawn(impl_reclaim_cycles_from_individual_canisters_and_send_to_platform_orchestrator())
}

async fn impl_reclaim_cycles_from_individual_canisters_and_send_to_platform_orchestrator() {
    let canister_ids: Vec<Principal> = CANISTER_DATA.with_borrow(|canister_data| {
        let mut canister_ids: Vec<Principal> = canister_data
            .user_principal_id_to_canister_id_map
            .clone()
            .into_values()
            .collect();

        canister_ids.extend(canister_data.available_canisters.iter());

        canister_ids
    });

    let reclaim_cycles_from_canister_futures = canister_ids
        .into_iter()
        .map(|canister_id| call::<_, ()>(canister_id, "return_cycles_to_user_index_canister", ()));
    run_task_concurrently(reclaim_cycles_from_canister_futures, 10, |_| {}, || false).await;
}
