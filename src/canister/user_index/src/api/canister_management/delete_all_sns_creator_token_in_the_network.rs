use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::{
    permissions::is_caller_controller_or_global_admin, task::run_task_concurrently,
};

use crate::{
    util::types::registered_individual_user_canister::RegisteredIndividualUserCanister,
    CANISTER_DATA,
};

#[update(guard = "is_caller_controller_or_global_admin")]
pub fn delete_all_sns_creator_token_in_the_network() {
    let individual_user_canisters: Vec<Principal> = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .user_principal_id_to_canister_id_map
            .values()
            .copied()
            .collect()
    });

    let delete_token_tasks =
        individual_user_canisters
            .into_iter()
            .map(|individual_canister_id| async move {
                let individual_canister =
                    RegisteredIndividualUserCanister::new(individual_canister_id)?;
                individual_canister.delete_all_sns_creator_token().await
            });

    ic_cdk::spawn(run_task_concurrently(
        delete_token_tasks,
        10,
        |_| {},
        || false,
    ));
}
