use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller;

use crate::util::types::individual_user_canister::IndividualUserCanister;

#[update(guard = "is_caller_controller")]
pub async fn make_individual_canister_logs_public(
    individual_canister: Principal,
) -> Result<(), String> {
    let individual_canister = IndividualUserCanister::new(individual_canister)?;
    individual_canister
        .make_individual_canister_logs_public()
        .await
}
