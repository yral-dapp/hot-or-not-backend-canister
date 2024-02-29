use ic_cdk::call;
use ic_cdk_macros::update;
use shared_utils::common::utils::task::run_task_concurrently;

use crate::CANISTER_DATA;

#[update]
fn reclaim_cycles_from_individual_canisters() {
    ic_cdk::spawn(impl_reclaim_cycles_from_individual_canisters_and_send_to_plaform_orchestrator())
}

async fn impl_reclaim_cycles_from_individual_canisters_and_send_to_plaform_orchestrator() {
    let canister_ids = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.user_principal_id_to_canister_id_map.clone().into_values()
    });

    let relcaim_cycles_from_canister_futures = canister_ids.map(|canister_id| {
        call::<_ , ()>(canister_id, "return_cycles_to_user_index_canister", ())
    });
    run_task_concurrently(relcaim_cycles_from_canister_futures, 10, |_| {}, || false).await;
}
