use candid::Principal;
use ic_stable_memory::utils::ic_types::SPrincipal;
use post_cache_lib::CanisterData;
use shared_utils::access_control::{self, UserAccessRole};

use crate::CANISTER_DATA;

#[ic_cdk_macros::update]
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
    access_control::remove_role_from_principal_id_v1(
        &mut canister_data.access_control_map,
        SPrincipal(principal_id),
        role,
        api_caller,
    );
}

#[cfg(test)]
mod test {
    use ic_stable_memory::utils::ic_types::SPrincipal;
    use post_cache_lib::CanisterData;
    use shared_utils::access_control::UserAccessRole;
    use test_utils::setup::test_constants::{
        get_alice_principal_id, get_bob_principal_id, get_global_super_admin_principal_id,
    };

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
        canister_data.access_control_map.insert(
            SPrincipal(get_alice_principal_id().0),
            vec![UserAccessRole::ProfileOwner],
        );

        // * removing role as bob from alice should not work
        let principal_id = get_alice_principal_id().0;
        let api_caller = get_bob_principal_id().0;
        super::update_user_remove_role_impl(
            UserAccessRole::ProfileOwner,
            principal_id,
            &mut canister_data,
            api_caller,
        );
        assert_eq!(
            canister_data
                .access_control_map
                .get(&SPrincipal(principal_id)),
            Some(&vec![UserAccessRole::ProfileOwner])
        );

        // * removing role as super admin from alice should work
        let principal_id = get_alice_principal_id().0;
        let api_caller = get_global_super_admin_principal_id().0;
        super::update_user_remove_role_impl(
            UserAccessRole::ProfileOwner,
            principal_id,
            &mut canister_data,
            api_caller,
        );
        assert_eq!(
            canister_data
                .access_control_map
                .get(&SPrincipal(principal_id)),
            Some(&vec![])
        );
    }
}
