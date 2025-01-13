use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::{
    types::known_principal::KnownPrincipalType, utils::permissions::is_caller_controller,
};

use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA};

#[update(guard = "is_caller_controller")]
fn update_well_known_principal(known_principal_type: KnownPrincipalType, value: Principal) {
    notify_to_recharge_canister();
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .known_principal_ids
            .insert(known_principal_type, value);
    })
}
