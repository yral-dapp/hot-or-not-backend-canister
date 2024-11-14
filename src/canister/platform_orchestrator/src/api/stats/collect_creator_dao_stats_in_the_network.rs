use ic_cdk::notify;
use ic_cdk_macros::update;

use crate::{guard::is_caller::is_caller_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_global_admin_or_controller")]
pub fn collect_creator_dao_stats_in_the_network() {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .all_subnet_orchestrator_canisters_list
            .iter()
            .for_each(|subnet_orchestrator_canister_id| {
                match notify(
                    *subnet_orchestrator_canister_id,
                    "collect_creator_dao_stats_in_the_network",
                    (),
                ) {
                    Err(e) => {
                        ic_cdk::println!(
                            "Failed to collect data from {:?}. Error {:?}",
                            subnet_orchestrator_canister_id,
                            e
                        )
                    }
                    _ => {}
                }
            });
    })
}
