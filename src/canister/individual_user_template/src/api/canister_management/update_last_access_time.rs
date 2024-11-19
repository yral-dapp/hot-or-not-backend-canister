use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::common::utils::system_time::get_current_system_time_from_ic;

use crate::{
    util::subnet_orchestrator::{self, SubnetOrchestrator},
    CANISTER_DATA,
};

#[update]
fn update_last_access_time() -> Result<String, String> {
    let profile_owner =
        CANISTER_DATA.with_borrow(|canister_data| canister_data.profile.principal_id.unwrap());
    if profile_owner != caller() {
        return Err("Unauthorized".into());
    }

    update_last_canister_functionality_access_time();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .session_type
            .ok_or(String::from("Canister not yet assigned"))?;
        canister_data.last_access_time = Some(get_current_system_time_from_ic());
        Ok("Success".into())
    })
}

#[update]
pub fn update_last_canister_functionality_access_time() {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.last_canister_functionality_access_time =
            Some(get_current_system_time_from_ic());
    });

    if let Ok(subnet_orchestrator) = SubnetOrchestrator::new() {
        subnet_orchestrator.receive_cycles_from_subnet_orchestrator();
    }
}
