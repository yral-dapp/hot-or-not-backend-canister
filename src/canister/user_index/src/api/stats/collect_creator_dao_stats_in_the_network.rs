use std::collections::HashMap;

use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    common::utils::{permissions::is_caller_controller, task::run_task_concurrently},
    types::creator_dao_stats::IndividualUserCreatorDaoEntry,
};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller")]
pub async fn collect_creator_dao_stats_in_the_network(
) -> Result<HashMap<Principal, IndividualUserCreatorDaoEntry>, String> {
    let individual_user_canisters: Vec<Principal> = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .user_principal_id_to_canister_id_map
            .values()
            .copied()
            .collect()
    });

    let mut creator_dao_stats_according_to_user_principal =
        HashMap::<Principal, IndividualUserCreatorDaoEntry>::new();

    let creator_dao_stats =
        individual_user_canisters
            .iter()
            .map(|individual_user_canister_canister_id| async {
                let creator_dao_stats_result =
                    ic_cdk::call::<_, (Result<IndividualUserCreatorDaoEntry, String>,)>(
                        *individual_user_canister_canister_id,
                        "send_creator_dao_stats_to_subnet_orchestrator",
                        (),
                    )
                    .await
                    .map_err(|e| e.1)
                    .and_then(|val| val.0);

                creator_dao_stats_result
            });

    let result_callback =
        |creator_dao_entry_result: Result<IndividualUserCreatorDaoEntry, String>| {
            match creator_dao_entry_result {
                Ok(creator_dao_entry) => {
                    creator_dao_stats_according_to_user_principal
                        .insert(creator_dao_entry.individual_profile_id, creator_dao_entry);
                }
                Err(e) => {
                    ic_cdk::println!(
                        "Error retrieving creator dao stats from individual canister. {}",
                        e
                    );
                }
            }
        };

    run_task_concurrently(creator_dao_stats, 10, result_callback, || false).await;

    Ok(creator_dao_stats_according_to_user_principal)
}
