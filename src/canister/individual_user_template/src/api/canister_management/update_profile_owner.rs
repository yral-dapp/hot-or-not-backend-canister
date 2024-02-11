use candid::Principal;
use ic_cdk::api::{self, is_controller};

use crate::CANISTER_DATA;


#[ic_cdk::update]
#[candid::candid_method]
pub async fn update_profile_owner(user_id: Option<Principal>) -> Result<(), String>{

    if !is_controller(&api::caller()) {
        return Err("UnAuthorised Access".into());
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        if canister_data.profile.principal_id.is_some() {
            return Err("Canister Already has a profile Owner".into());
        }
        canister_data.profile.principal_id = user_id;
        Ok(())
    })
}