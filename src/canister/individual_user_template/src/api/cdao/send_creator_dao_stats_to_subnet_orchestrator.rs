use std::collections::HashSet;

use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    common::utils::permissions::is_caller_controller,
    types::creator_dao_stats::IndividualUserCreatorDaoEntry,
};

use crate::{
    util::{cycles::notify_to_recharge_canister, subnet_orchestrator::SubnetOrchestrator},
    CANISTER_DATA,
};

#[update(guard = "is_caller_controller")]
fn send_creator_dao_stats_to_subnet_orchestrator() -> Result<IndividualUserCreatorDaoEntry, String>
{
    notify_to_recharge_canister();

    CANISTER_DATA.with_borrow(|canister_data| {
        let root_canisters: HashSet<Principal> = canister_data
            .cdao_canisters
            .iter()
            .map(|deployed_canisters| deployed_canisters.root)
            .collect();

        Ok(IndividualUserCreatorDaoEntry {
            individual_profile_id: canister_data.profile.principal_id.unwrap(),
            deployed_canisters: root_canisters,
        })
    })
}
