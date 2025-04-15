use shared_utils::{
    canister_specific::individual_user_template::types::ml_data::WatchHistoryItem,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use ic_cdk_macros::update;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_watch_history(watch_history_item: WatchHistoryItem) -> Result<String, String> {
    CANISTER_DATA.with(|canister_data| {
        let mut canister_data = canister_data.borrow_mut();
        canister_data.watch_history.insert(watch_history_item, ());

        // keep removing oldest items until the len is less than or equal to 3000
        while canister_data.watch_history.len() > 3000 {
            canister_data.watch_history.pop_first();
        }
    });

    Ok("Success".into())
}
