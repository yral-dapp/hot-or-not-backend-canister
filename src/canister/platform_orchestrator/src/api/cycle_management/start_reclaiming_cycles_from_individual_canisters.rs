
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller")]
fn start_reclaiming_cycles_from_individual_canisters() -> Result<String, String>{
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.all_subnet_orchestrator_canisters_list.iter().for_each(|subnet_orchestrator_id| {
            ic_cdk::notify(*subnet_orchestrator_id, "reclaim_cycles_from_individual_canisters", ()).unwrap();
        });
   });
   Ok("Success".into())
}