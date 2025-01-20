use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::session::SessionType,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA, PUMP_N_DUMP};

use super::update_last_access_time::update_last_canister_functionality_access_time;

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_session_type(session_type: SessionType) -> Result<String, String> {
    notify_to_recharge_canister();
    update_last_canister_functionality_access_time();

    let res = CANISTER_DATA.with_borrow_mut(|canister_data| {
        if matches!(&canister_data.session_type, Some(SessionType::RegisteredSession)) {
            return Err("Session Already marked as Registered Session".to_string());
        }
        canister_data.session_type = Some(session_type);

        Ok("Success".to_string())
    })?;

    if session_type != SessionType::RegisteredSession {
        return Ok(res);
    }

    PUMP_N_DUMP.with_borrow_mut(|pd| {
        pd.game_only_balance += pd.onboarding_reward.clone();
    });

    Ok(res)
}
