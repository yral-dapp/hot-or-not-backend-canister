use candid::Principal;
use ic_stable_memory::utils::ic_types::SPrincipal;
use shared_utils::access_control::{add_role_to_principal_id_v1, UserAccessRole};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk_macros::update]
#[candid::candid_method(update)]
fn update_user_add_role(role: UserAccessRole, principal_id: Principal) {
    let api_caller = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();
        update_user_add_role_impl(role, principal_id, &mut canister_data, api_caller);
    });
}

fn update_user_add_role_impl(
    role: UserAccessRole,
    principal_id: Principal,
    canister_data: &mut CanisterData,
    api_caller: Principal,
) {
    add_role_to_principal_id_v1(
        &mut canister_data.access_control_map,
        SPrincipal(principal_id),
        role,
        api_caller,
    );
}

#[cfg(test)]
mod test {
    use ic_stable_memory::utils::ic_types::SPrincipal;
    use shared_utils::access_control::{self, UserAccessRole};
    use test_utils::setup::test_constants::{
        get_alice_principal_id, get_bob_principal_id, get_global_super_admin_principal_id,
    };

    use crate::data_model::CanisterData;

    #[test]
    fn test_update_user_add_role_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.access_control_map.insert(
            SPrincipal(get_global_super_admin_principal_id().0),
            vec![
                UserAccessRole::CanisterAdmin,
                UserAccessRole::CanisterController,
            ],
        );

        // * adding role as super admin to alice should work
        let principal_id = get_alice_principal_id().0;
        let api_caller = get_global_super_admin_principal_id().0;
        super::update_user_add_role_impl(
            UserAccessRole::ProfileOwner,
            principal_id,
            &mut canister_data,
            api_caller,
        );

        let user_roles = access_control::get_roles_for_principal_id_v1(
            &canister_data.access_control_map,
            SPrincipal(principal_id),
        );
        assert!(user_roles.contains(&UserAccessRole::ProfileOwner));

        // * adding role as bob to alice should fail
        let principal_id = get_alice_principal_id().0;
        let api_caller = get_bob_principal_id().0;
        super::update_user_add_role_impl(
            UserAccessRole::CanisterController,
            principal_id,
            &mut canister_data,
            api_caller,
        );

        let user_roles = access_control::get_roles_for_principal_id_v1(
            &canister_data.access_control_map,
            SPrincipal(principal_id),
        );
        assert!(!user_roles.contains(&UserAccessRole::CanisterController));
    }
}
