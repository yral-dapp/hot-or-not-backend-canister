use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller;

use crate::util::types::registered_individual_user_canister::RegisteredIndividualUserCanister;

#[update(guard = "is_caller_controller")]
pub async fn make_individual_canister_logs_private(
    individual_canister_id: Principal,
) -> Result<(), String> {
    let individual_canister = RegisteredIndividualUserCanister::new(individual_canister_id)?;
    individual_canister
        .make_indvidual_canister_logs_private()
        .await
}
