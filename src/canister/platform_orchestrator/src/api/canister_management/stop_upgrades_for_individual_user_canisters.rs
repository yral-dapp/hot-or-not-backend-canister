use ic_cdk::{api::{call::CallResult, is_controller}, call, caller};
use ic_cdk_macros::update;

use crate::CANISTER_DATA;

#[update]
async fn stop_upgrades_for_individual_user_canisters() -> Result<String, String> {

    if !is_controller(&caller())  {
        return Err("Unauthorized".into())
    }

    let subnet_orchestrator_list = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.all_subnet_orchestrator_canisters_list.clone()
    });

    for subnet_orchestrator in subnet_orchestrator_list {
        let result: CallResult<()> = call(subnet_orchestrator, "set_permission_to_upgrade_individual_canisters", (false, )).await;
        result.map_err(|e| format!("failed to stop upgrades for {} {}", subnet_orchestrator, e.1))?;
    }

    Ok("Success".into())

}