use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::util::types::registered_individual_user_canister::RegisteredIndividualUserCanister;

#[update(guard = "is_caller_controller_or_global_admin")]
pub async fn delete_all_sns_creator_token_of_an_individual_canister(
    individual_canister_id: Principal,
) -> Result<(), String> {
    let individual_user_canister = RegisteredIndividualUserCanister::new(individual_canister_id)?;
    individual_user_canister
        .delete_all_sns_creator_token()
        .await
}
