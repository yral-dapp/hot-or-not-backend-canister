use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::session::SessionType,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use crate::CANISTER_DATA;

use super::update_last_access_time::update_last_canister_functionality_access_time;

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_session_type(session_type: SessionType) -> Result<String, String> {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let current_session_type = &mut canister_data.session_type;

        match current_session_type {
            Some(val) => {
                if *val == SessionType::RegisteredSession {
                    return Err("Session Already marked as Registered Session".into());
                }
                *val = session_type;
            }
            None => canister_data.session_type = Some(session_type),
        }

        Ok("Success".into())
    })
}
