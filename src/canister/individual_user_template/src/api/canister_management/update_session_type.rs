use std::borrow::Borrow;

use ic_cdk::{caller, id, notify};
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::session::SessionType,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA};

use super::update_last_access_time::update_last_canister_functionality_access_time;

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_session_type(session_type: SessionType) -> Result<String, String> {
    notify_to_recharge_canister();
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

        if session_type == SessionType::RegisteredSession {
            canister_data.pump_n_dump.game_only_balance += canister_data.pump_n_dump.onboarding_reward.clone();
        }

        Ok("Success".into())
    })
}
