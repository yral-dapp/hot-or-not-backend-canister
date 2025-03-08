use ic_cdk::caller;
use ic_cdk_macros::update;

use crate::util::types::registered_individual_user_canister::RegisteredIndividualUserCanister;

#[update]
async fn recharge_individual_user_canister() -> Result<(), String> {
    let individual_user_canister = RegisteredIndividualUserCanister::new(caller())?;
    individual_user_canister
        .recharge_individual_canister()
        .await
}
