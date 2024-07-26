use shared_utils::{
    canister_specific::individual_user_template::types::ml_data::WatchHistoryItem,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use ic_cdk_macros::update;

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_watch_history(watch_history_item: WatchHistoryItem) -> Result<String, String> {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data| {
        let mut canister_data = canister_data.borrow_mut();
        canister_data.watch_history.insert(watch_history_item, ());
    });

    Ok("Success".into())
}
