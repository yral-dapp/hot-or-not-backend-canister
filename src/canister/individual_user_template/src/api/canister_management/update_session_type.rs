use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::session::SessionType,
    common::utils::permissions::is_caller_controller_or_global_admin_v2,
};

use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA};

use super::update_last_access_time::update_last_canister_functionality_access_time;

#[update]
fn update_session_type(session_type: SessionType) -> Result<String, String> {
    notify_to_recharge_canister();
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        is_caller_controller_or_global_admin_v2(&canister_data.known_principal_ids)?;

        if matches!(&canister_data.session_type, Some(SessionType::RegisteredSession)) {
            return Err("Session Already marked as Registered Session".to_string());
        }
        canister_data.session_type = Some(session_type);

        Ok("Success".to_string())
    })
}
