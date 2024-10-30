use ic_cdk::api::{
    call::CallResult,
    management_canister::{
        main::{self, start_canister, stop_canister, InstallCodeArgument},
        provisional::CanisterIdRecord,
    },
};

pub async fn upgrade_canister_util(arg: InstallCodeArgument) -> CallResult<()> {
    let canister_id = arg.canister_id;
    stop_canister(CanisterIdRecord { canister_id }).await?;
    let install_code_result = main::install_code(arg).await;
    start_canister(CanisterIdRecord { canister_id }).await?;
    install_code_result
}
