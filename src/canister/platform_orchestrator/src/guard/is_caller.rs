use ic_cdk::{api::is_controller, caller};

use crate::CANISTER_DATA;

pub fn is_caller_global_admin() -> Result<(), String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let res = canister_data.platform_global_admins.contains(&caller());
        match res {
            true => Ok(()),
            false => Err("Unauthorized".into())
        }
    })
}

pub fn is_caller_global_admin_or_controller() -> Result<(), String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        match canister_data.platform_global_admins.contains(&caller()) || is_controller(&caller()) {
            true => Ok(()),
            false => Err("Unauthorized".into())
        }
    })
}

