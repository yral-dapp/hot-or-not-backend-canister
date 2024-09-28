use ic_cdk_macros::update;

use crate::CANISTER_DATA;

#[update]
fn update_profile_owner_for_individual_canisters() {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .all_subnet_orchestrator_canisters_list
            .iter()
            .for_each(|subnet_canster_id| {
                let _ = ic_cdk::notify(
                    *subnet_canster_id,
                    "update_profile_owner_for_individual_canisters",
                    (),
                );
            })
    })
}
