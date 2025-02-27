use candid::Principal;
use futures::{future::BoxFuture, FutureExt};
use ic_cdk::api::{
    call::CallResult,
    management_canister::{
        main::{self, start_canister, stop_canister, InstallCodeArgument},
        provisional::CanisterIdRecord,
    },
};

pub async fn upgrade_canister_util(arg: InstallCodeArgument) -> CallResult<()> {
    let canister_id = arg.canister_id;
    try_stopping_canister_with_retries(canister_id, 3).await?;
    let install_code_result = main::install_code(arg).await;
    start_canister(CanisterIdRecord { canister_id }).await?;
    install_code_result
}

fn try_stopping_canister_with_retries(
    canister_id: Principal,
    max_retries: u64,
) -> BoxFuture<'static, CallResult<()>> {
    async move {
        let stop_canister_result = stop_canister(CanisterIdRecord { canister_id }).await;

        match stop_canister_result {
            Ok(()) => Ok(()),
            Err(e) => {
                if max_retries > 0 {
                    Box::pin(try_stopping_canister_with_retries(
                        canister_id,
                        max_retries - 1,
                    ))
                    .await
                } else {
                    Err(e)
                }
            }
        }
    }
    .boxed()
}
