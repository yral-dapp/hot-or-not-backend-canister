use ic_cdk_macros::update;

use crate::{guard::is_caller::is_caller_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_global_admin_or_controller")]
fn start_reclaiming_cycles_from_individual_canisters() -> Result<String, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .subnet_orchestrators()
            .iter()
            .for_each(|subnet_orchestrator_id| {
                ic_cdk::notify(
                    *subnet_orchestrator_id,
                    "reclaim_cycles_from_individual_canisters",
                    (),
                )
                .unwrap();
            });
    });
    Ok("Success".into())
}
