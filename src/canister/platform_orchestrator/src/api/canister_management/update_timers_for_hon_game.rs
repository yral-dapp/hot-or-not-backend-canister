use ic_cdk::{api::call::CallResult, call};
use ic_cdk_macros::update;

use crate::{guard::is_caller::is_caller_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_global_admin_or_controller")]
async fn update_restart_timers_hon_game() -> Result<String, String> {
    let subnet_orchestrator_list = CANISTER_DATA
        .with_borrow(|canister_data| canister_data.all_subnet_orchestrator_canisters_list.clone());

    for subnet_orchestrator in subnet_orchestrator_list {
        let result: CallResult<()> = call(
            subnet_orchestrator,
            "update_restart_timers_hon_game",
            (),
        )
        .await;
        result.map_err(|e| {
            format!(
                "failed to call update_restart_timers_hon_game for {} {}",
                subnet_orchestrator, e.1
            )
        })?;
    }

    Ok("Success".into())
}
