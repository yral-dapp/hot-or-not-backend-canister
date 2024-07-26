use shared_utils::{
    canister_specific::individual_user_template::types::ml_data::{
        SuccessHistoryItem, WatchHistoryItem,
    },
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use ic_cdk_macros::query;

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[query(guard = "is_caller_controller_or_global_admin")]
fn get_watch_history() -> Result<Vec<WatchHistoryItem>, String> {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data| {
        let canister_data = canister_data.borrow();
        Ok(canister_data
            .watch_history
            .iter()
            .map(|(k, _)| k.clone())
            .collect())
    })
}

#[query(guard = "is_caller_controller_or_global_admin")]
fn get_success_history() -> Result<Vec<SuccessHistoryItem>, String> {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data| {
        let canister_data = canister_data.borrow();
        Ok(canister_data
            .success_history
            .iter()
            .map(|(k, _)| k.clone())
            .collect())
    })
}
