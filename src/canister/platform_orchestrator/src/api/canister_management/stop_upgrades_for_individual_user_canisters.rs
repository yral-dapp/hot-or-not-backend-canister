use ic_cdk::{api::{call::CallResult, is_controller}, call, caller};
use ic_cdk_macros::update;

use crate::{guard::is_caller::is_caller_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_global_admin_or_controller")]
async fn stop_upgrades_for_individual_user_canisters() -> Result<String, String> {

    let subnet_orchestrator_list = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.all_subnet_orchestrator_canisters_list.clone()
    });

    for subnet_orchestrator in subnet_orchestrator_list {
        let result: CallResult<()> = call(subnet_orchestrator, "set_permission_to_upgrade_individual_canisters", (false, )).await;
        result.map_err(|e| format!("failed to stop upgrades for {} {}", subnet_orchestrator, e.1))?;
    }

    Ok("Success".into())
}