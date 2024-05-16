use candid::Principal;
use ic_cdk::api::{self, is_controller};
use ic_cdk_macros::update;

use crate::CANISTER_DATA;

use super::update_last_access_time::update_last_canister_functionality_access_time;

#[update]
pub async fn update_profile_owner(user_id: Option<Principal>) -> Result<(), String> {
    if !is_controller(&api::caller()) {
        return Err("Unauthorised".into());
    }

    update_last_canister_functionality_access_time();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        if canister_data.profile.principal_id.is_some() {
            return Err("Canister Already has a profile Owner".into());
        }
        canister_data.profile.principal_id = user_id;
        Ok(())
    })
}
