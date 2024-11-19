use std::collections::HashSet;

use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller;

use crate::{util::subnet_orchestrator::SubnetOrchestrator, CANISTER_DATA};

#[update(guard = "is_caller_controller")]
fn send_creator_dao_stats_to_subnet_orchestrator() {
    let root_canisters: HashSet<Principal> = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .cdao_canisters
            .iter()
            .map(|deployed_canisters| deployed_canisters.root)
            .collect()
    });

    let subnet_orchestrator = SubnetOrchestrator::new();

    if let Ok(subnet_orchestrator) = subnet_orchestrator {
        match subnet_orchestrator.send_creator_dao_stats(root_canisters) {
            Ok(()) => {}
            Err(e) => {
                ic_cdk::println!("Error sending creator dao stats to subnet orchestrator {e}")
            }
        }
    } else {
        ic_cdk::println!("Subnet Orchestrator canister id not found");
    }
}
