use ic_cdk_macros::update;
use shared_utils::{canister_specific::individual_user_template::types::session::SessionType, common::utils::permissions::is_caller_controller_or_global_admin};

use crate::CANISTER_DATA;

#[update(guard="is_caller_controller_or_global_admin")]
fn update_session_type(session_type: Option<SessionType>) -> String {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.session_type = session_type;
    });

    String::from("Success")
}