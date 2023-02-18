use candid::Principal;
use shared_utils::access_control::{self, UserAccessRole};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk::update]
#[candid::candid_method(update)]
fn update_user_remove_role(role: UserAccessRole, principal_id: Principal) {
    let api_caller = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();
        update_user_remove_role_impl(role, principal_id, &mut canister_data, api_caller);
    });
}

fn update_user_remove_role_impl(
    role: UserAccessRole,
    principal_id: Principal,
    canister_data: &mut CanisterData,
    api_caller: Principal,
) {
    access_control::remove_role_from_principal_id_v2(
        &mut canister_data.access_control_map,
        principal_id,
        role,
        api_caller,
    );
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id_v1, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id,
    };

    use super::*;

    #[test]
    fn test_update_user_add_role_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.access_control_map.insert(
            get_global_super_admin_principal_id_v1(),
            vec![
                UserAccessRole::CanisterAdmin,
                UserAccessRole::CanisterController,
            ],
        );
        canister_data.access_control_map.insert(
            get_mock_user_alice_principal_id(),
            vec![UserAccessRole::ProfileOwner],
        );

        // * removing role as bob from alice should not work
        let principal_id = get_mock_user_alice_principal_id();
        let api_caller = get_mock_user_bob_principal_id();
        super::update_user_remove_role_impl(
            UserAccessRole::ProfileOwner,
            principal_id,
            &mut canister_data,
            api_caller,
        );
        assert_eq!(
            canister_data.access_control_map.get(&principal_id),
            Some(&vec![UserAccessRole::ProfileOwner])
        );

        // * removing role as super admin from alice should work
        let principal_id = get_mock_user_alice_principal_id();
        let api_caller = get_global_super_admin_principal_id_v1();
        super::update_user_remove_role_impl(
            UserAccessRole::ProfileOwner,
            principal_id,
            &mut canister_data,
            api_caller,
        );
        assert_eq!(
            canister_data.access_control_map.get(&principal_id),
            Some(&vec![])
        );
    }
}
