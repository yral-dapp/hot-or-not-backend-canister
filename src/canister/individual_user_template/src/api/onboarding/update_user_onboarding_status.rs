use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::common::utils::{
    permissions::is_caller_controller, system_time::get_current_system_time_from_ic,
};

use crate::CANISTER_DATA;

#[update]
#[update(guard = "is_caller_controller")]
fn update_user_onboarding_status() -> Result<(), String> {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.onboarding_status = Some(true);
        Ok(())
    })
}
