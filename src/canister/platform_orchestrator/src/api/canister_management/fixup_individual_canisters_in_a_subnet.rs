use candid::Principal;
use ic_cdk::update;

use crate::{guard::is_caller::is_caller_global_admin_or_controller, utils::registered_subnet_orchestrator::RegisteredSubnetOrchestrator};

#[update(guard = "is_caller_global_admin_or_controller")]
async fn fixup_individual_canisters_in_a_subnet(subnet_orchestrator: Principal) -> Result<(), String>{
    let resgistered_subnet_orchestrator = RegisteredSubnetOrchestrator::new(subnet_orchestrator)?;

    resgistered_subnet_orchestrator.fixup_individual_cansiters_mapping().await

}