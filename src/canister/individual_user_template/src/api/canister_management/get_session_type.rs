use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::session::SessionType;

use crate::CANISTER_DATA;

#[query]
fn get_session_type() -> Result<SessionType, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.session_type.ok_or(String::from("Canister has not been assigned yet"))
    })
}