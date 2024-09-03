use ic_cdk_macros::query;

use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, CANISTER_DATA,
};

#[query]
fn get_user_onboarding_status() -> bool {
    update_last_canister_functionality_access_time();

    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .onboarding_status
            .unwrap_or(false)
    })
}
