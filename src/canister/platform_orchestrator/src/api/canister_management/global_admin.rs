use candid::Principal;
use ic_cdk_macros::{query, update};

use crate::{guard::is_caller::is_caller_platform_global_admin_or_controller, CANISTER_DATA};

#[update(guard = "is_caller_platform_global_admin_or_controller")]
fn add_principal_as_global_admin(id: Principal) {
    CANISTER_DATA.with_borrow_mut(|canister_data| canister_data.platform_global_admins.insert(id));
}

#[update(guard = "is_caller_platform_global_admin_or_controller")]
fn remove_principal_from_global_admins(id: Principal) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.platform_global_admins.remove(&id);
    })
}

#[query]
fn get_all_global_admins() -> Vec<Principal> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .platform_global_admins
            .clone()
            .into_iter()
            .collect()
    })
}
