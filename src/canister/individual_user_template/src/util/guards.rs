use ic_cdk::caller;

use crate::CANISTER_DATA;

pub(crate) fn is_caller_profile_owner() -> Result<(), String> {
    CANISTER_DATA.with_borrow(|canister_data| match canister_data.profile.principal_id {
        Some(principal_id) if principal_id == caller() => Ok(()),
        _ => Err("Unauthorized".to_owned()),
    })
}
