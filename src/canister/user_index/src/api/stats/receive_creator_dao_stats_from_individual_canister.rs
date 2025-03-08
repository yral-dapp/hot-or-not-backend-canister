use std::collections::HashSet;

use ic_cdk::{caller, notify};
use ic_cdk_macros::update;

use candid::Principal;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::{
    util::types::registered_individual_user_canister::RegisteredIndividualUserCanister,
    CANISTER_DATA,
};

#[update]
pub fn receive_creator_dao_stats_from_individual_canister(
    root_canister_ids: HashSet<Principal>,
) -> Result<(), String> {
    let individual_user = RegisteredIndividualUserCanister::new(caller())?;

    let platform_orchestrator_canister_id = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .configuration
            .known_principal_ids
            .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
            .copied()
    });

    let platform_orchestrator_canister_id =
        platform_orchestrator_canister_id.ok_or("Platform Orchestrator Canister Id not found")?;

    notify(
        platform_orchestrator_canister_id,
        "receive_creator_dao_stats_from_subnet_orchestrator",
        (individual_user.profile_id, root_canister_ids),
    )
    .map_err(|e| format!("failed to notify platform orchestrator {:?}", e))
}
