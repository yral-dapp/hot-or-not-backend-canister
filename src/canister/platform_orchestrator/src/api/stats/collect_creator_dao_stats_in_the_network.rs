use std::{collections::HashMap, hash::Hash};

use candid::Principal;
use ic_cdk::notify;
use ic_cdk_macros::update;
use shared_utils::{
    common::utils::task::run_task_concurrently,
    types::creator_dao_stats::{self, IndividualUserCreatorDaoEntry},
};

use crate::{guard::is_caller::is_caller_platform_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
pub fn collect_creator_dao_stats_in_the_network() {
    let subnet_orchestrators = CANISTER_DATA
        .with_borrow(|canister_data| canister_data.all_subnet_orchestrator_canisters_list.clone());

    let creator_dao_stats_task =
        subnet_orchestrators
            .into_iter()
            .map(|subnet_orchestrator_canister_id| async move {
                let creator_dao_entry_call_result = ic_cdk::call::<
                    _,
                    (Result<HashMap<Principal, IndividualUserCreatorDaoEntry>, String>,),
                >(
                    subnet_orchestrator_canister_id,
                    "collect_creator_dao_stats_in_the_network",
                    (),
                )
                .await
                .map_err(|e| e.1)
                .and_then(|res| res.0);

                creator_dao_entry_call_result
            });

    let creator_dao_stats_result_callback = |creator_dao_stats_result: Result<
        HashMap<Principal, IndividualUserCreatorDaoEntry>,
        String,
    >| {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            if let Ok(creator_dao_stats) = creator_dao_stats_result {
                creator_dao_stats.into_iter().for_each(
                    |(individual_user_profile_id, individual_user_creator_dao_entry)| {
                        individual_user_creator_dao_entry
                            .deployed_canisters
                            .into_iter()
                            .for_each(|deployed_canister| {
                                canister_data.creator_dao_stats.insert_new_entry(
                                    individual_user_profile_id,
                                    deployed_canister,
                                );
                            });
                    },
                );
            }
        })
    };

    ic_cdk::spawn(run_task_concurrently(
        creator_dao_stats_task,
        5,
        creator_dao_stats_result_callback,
        || false,
    ));
}
