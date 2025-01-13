use ic_cdk::{call, caller};
use ic_cdk_macros::update;
use shared_utils::canister_specific::individual_user_template::types::{
    profile::{self, UserCanisterDetails},
    session::SessionType,
};

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    util::cycles::notify_to_recharge_canister, CANISTER_DATA,
};

#[update]
async fn update_referrer_details(referrer: UserCanisterDetails) -> Result<String, String> {
    notify_to_recharge_canister();

    let profile_owner =
        CANISTER_DATA.with_borrow_mut(|canister_data| canister_data.profile.principal_id);
    if profile_owner.is_none() || !profile_owner.unwrap().eq(&caller()) {
        return Err("Unauthorized".into());
    }

    update_last_canister_functionality_access_time();

    let (canister_session_type_result,): (Result<SessionType, String>,) =
        call(referrer.user_canister_id, "get_session_type", ())
            .await
            .map_err(|e| e.1)?;
    let canister_session_type = canister_session_type_result?;

    if canister_session_type == SessionType::AnonymousSession {
        return Err("referrer not signed up".into());
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let profile = &mut canister_data.profile;

        if profile.referrer_details.is_some() {
            return Err("Referrer is already set".into());
        }

        profile.referrer_details = Some(referrer);

        Ok("Success".into())
    })
}
