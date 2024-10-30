use crate::{
    api::canister_management::update_last_access_time::update_last_canister_functionality_access_time,
    data_model::CanisterData, CANISTER_DATA,
};
use ic_cdk_macros::{query, update};
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_user_propensity(user_propensity: f64) -> Result<String, String> {
    update_last_canister_functionality_access_time();

    let _ = CANISTER_DATA.with(|canister_data| {
        let mut canister_data = canister_data.borrow_mut();

        canister_data.user_propensity = user_propensity;
    });

    Ok("Success".into())
}

#[query]
fn get_user_propensity() -> f64 {
    CANISTER_DATA.with(|canister_data| {
        let canister_data = canister_data.borrow();

        canister_data.user_propensity
    })
}
