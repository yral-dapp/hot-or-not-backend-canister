use ic_cdk::api;
use ic_cdk_macros::update;
use shared_utils::canister_specific::individual_user_template::types::device_id::DeviceIdentity;

use crate::{
    data_model::CanisterData,
    CANISTER_DATA
};

/// #### Access Control
/// Only the user whose profile details are stored in this canister can add the device identity.
#[update]
fn add_device_id(identity_token: String) -> Result<bool, ()> {
    // * access control
    let current_caller = ic_cdk::caller();

    let device_id = DeviceIdentity {
        device_id: identity_token,
        timestamp: api::time() / 1_000_000,
    };

    let response = CANISTER_DATA.with(|canister_data_ref_cell| {
        add_device_id_to_memory(
            &mut canister_data_ref_cell.borrow_mut(),
            device_id
        )
    });

    if response {
        Ok(true)
    } else {
        Err(())
    }
}

fn add_device_id_to_memory(canister_data: &mut CanisterData, device_id: DeviceIdentity) -> bool {
    canister_data.device_identities.push(device_id);
    true
}