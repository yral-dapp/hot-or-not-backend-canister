use candid::Principal;
use ic_cdk::api::{self, is_controller};
use ic_cdk_macros::update;

use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA};

#[update]
pub async fn update_profile_owner(user_id: Option<Principal>) -> Result<(), String> {
    notify_to_recharge_canister();
    if !is_controller(&api::caller()) {
        return Err("Unauthorised".into());
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        if canister_data.profile.principal_id.is_some() {
            return Err("Canister Already has a profile Owner".into());
        }
        canister_data.profile.principal_id = user_id;
        Ok(())
    })
}
