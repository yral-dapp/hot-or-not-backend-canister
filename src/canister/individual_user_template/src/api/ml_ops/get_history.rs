use shared_utils::canister_specific::individual_user_template::types::ml_data::{
    SuccessHistoryItemV1, WatchHistoryItem,
};

use ic_cdk_macros::query;

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[query]
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

#[query]
fn get_success_history() -> Result<Vec<SuccessHistoryItemV1>, String> {
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
