use ic_cdk_macros::update;

use crate::{guard::is_caller::is_caller_platform_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn start_reclaiming_cycles_from_subnet_orchestrator_canister() -> String {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .all_subnet_orchestrator_canisters_list
            .iter()
            .for_each(|subnet_orchestrator_id| {
                ic_cdk::notify(
                    *subnet_orchestrator_id,
                    "return_cycles_to_platform_orchestrator_canister",
                    (),
                )
                .unwrap();
            });
    });

    String::from("Success")
}
