
use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::{canister_specific::individual_user_template::types::session::SessionType, common::utils::permissions::is_caller_global_admin};
use shared_utils::common::utils::system_time::get_current_system_time_from_ic;

use crate::CANISTER_DATA;

#[update]
fn update_last_access_time() -> Result<String, String> {

    let profile_owner = CANISTER_DATA.with_borrow(|canister_data| canister_data.profile.principal_id.unwrap());
    if profile_owner != caller() {
        return Err("Unauthorized".into())
    }
    
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let session_type = canister_data.session_type.ok_or(String::from("User is not yet registered"))?;
        match session_type {
            SessionType::RegisteredSession => {
                canister_data.last_access_time = Some(get_current_system_time_from_ic());
                Ok("Success".into())
            },
            SessionType::AnonymousSession => {
                Err(String::from("User is not yet registered"))
            }
        }
    })
}