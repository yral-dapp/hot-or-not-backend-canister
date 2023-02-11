use candid::Principal;
use shared_utils::access_control::{self, UserAccessRole};

use crate::{data::CanisterData, CANISTER_DATA};

#[ic_cdk_macros::update]
#[candid::candid_method(update)]
fn toggle_signups_enabled() {
    let api_caller = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        toggle_signups_enabled_impl(api_caller, &mut canister_data);
    });
}

fn toggle_signups_enabled_impl(caller: Principal, canister_data: &mut CanisterData) {
    if !access_control::does_principal_have_role_v2(
        &canister_data.access_control_list,
        UserAccessRole::CanisterAdmin,
        caller,
    ) {
        return;
    };

    canister_data.signups_enabled = !canister_data.signups_enabled;
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use shared_utils::access_control::UserAccessRole;
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id_v1, get_mock_user_alice_principal_id,
    };

    use crate::data::CanisterData;

    use super::*;

    #[test]
    fn test_toggle_signups_enabled_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.access_control_list = HashMap::new();
        canister_data.access_control_list.insert(
            get_global_super_admin_principal_id_v1(),
            vec![UserAccessRole::CanisterAdmin],
        );
        let admin_caller = get_global_super_admin_principal_id_v1();
        // super admin should be allowed to toggle
        toggle_signups_enabled_impl(admin_caller, &mut canister_data);
        assert_eq!(canister_data.signups_enabled, true);
        toggle_signups_enabled_impl(admin_caller, &mut canister_data);
        assert_eq!(canister_data.signups_enabled, false);

        // non super admin should not be allowed to toggle
        let non_admin_caller = get_mock_user_alice_principal_id();
        toggle_signups_enabled_impl(non_admin_caller, &mut canister_data);
        assert_eq!(canister_data.signups_enabled, false);
    }
}
