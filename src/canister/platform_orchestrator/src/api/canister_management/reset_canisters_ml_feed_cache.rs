use ic_cdk::{api::call::CallResult, call};
use ic_cdk_macros::update;

use crate::{guard::is_caller::is_caller_platform_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn reset_canisters_ml_feed_cache() -> Result<String, String> {
    let subnet_orchestrator_list = CANISTER_DATA
        .with_borrow(|canister_data| canister_data.all_subnet_orchestrator_canisters_list.clone());

    for subnet_orchestrator in subnet_orchestrator_list {
        let result: CallResult<()> = call(
            subnet_orchestrator,
            "reset_user_canisters_ml_feed_cache",
            (),
        )
        .await;
        result.map_err(|e| {
            format!(
                "failed to call reset_user_canisters_ml_feed_cache for {} {}",
                subnet_orchestrator, e.1
            )
        })?;
    }

    Ok("Success".into())
}
