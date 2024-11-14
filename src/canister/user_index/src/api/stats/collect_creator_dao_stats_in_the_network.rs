use ic_cdk::notify;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller")]
pub fn collect_creator_dao_stats_in_the_network() {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .user_principal_id_to_canister_id_map
            .iter()
            .for_each(|(_, user_canister)| {
                match notify(
                    *user_canister,
                    "send_creator_dao_stats_to_subnet_orchestrator",
                    (),
                ) {
                    Err(e) => {
                        ic_cdk::println!(
                            "Failed to collect data from {:?}. Error {:?}",
                            user_canister,
                            e
                        )
                    }
                    _ => {}
                }
            });
    })
}
