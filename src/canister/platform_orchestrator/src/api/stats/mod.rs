use ic_cdk_macros::update;
use std::collections::HashSet;

use candid::Principal;
use ic_cdk::caller;

use crate::{utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator, CANISTER_DATA};

#[update]
pub fn receive_creator_dao_stats_from_subnet_orchestrator(
    individual_user_profile_id: Principal,
    root_canister_ids: HashSet<Principal>,
) -> Result<(), String> {
    let _registered_subnet_orchestrator = RegisteredSubnetOrchestrator::new(caller())?;
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.add_creator_dao_stats_recieved_from_subnet_orchestrator(
            individual_user_profile_id,
            root_canister_ids,
        );
    });

    Ok(())
}
