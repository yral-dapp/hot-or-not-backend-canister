use shared_utils::{
    canister_specific::individual_user_template::types::ml_data::SuccessHistoryItemV1,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use ic_cdk_macros::update;

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    CANISTER_DATA,
};

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_success_history(success_history_item: SuccessHistoryItemV1) -> Result<String, String> {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with(|canister_data| {
        let mut canister_data = canister_data.borrow_mut();
        canister_data
            .success_history
            .insert(success_history_item, ());

        // keep removing oldest items until the len is less than or equal to 3000
        while canister_data.success_history.len() > 3000 {
            canister_data.success_history.pop_first();
        }
    });

    Ok("Success".into())
}
