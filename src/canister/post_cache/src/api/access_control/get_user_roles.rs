use candid::Principal;
use ic_stable_memory::utils::ic_types::SPrincipal;
use shared_utils::access_control::{self, UserAccessRole};

use crate::{data_model::CanisterData, CANISTER_DATA};

#[ic_cdk_macros::query]
#[candid::candid_method(query)]
fn get_user_roles(principal_id: Principal) -> Vec<UserAccessRole> {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        get_user_roles_impl(principal_id, &canister_data_ref_cell.borrow())
    })
}

fn get_user_roles_impl(
    principal_id: Principal,
    canister_data: &CanisterData,
) -> Vec<UserAccessRole> {
    access_control::get_roles_for_principal_id_v1(
        &canister_data.access_control_map,
        SPrincipal(principal_id),
    )
}

#[cfg(test)]
mod test {
    use ic_stable_memory::utils::ic_types::SPrincipal;
    use shared_utils::access_control::UserAccessRole;
    use test_utils::setup::test_constants::{
        get_alice_principal_id, get_global_super_admin_principal_id,
    };

    use crate::data_model::CanisterData;

    #[test]
    fn test_get_user_roles_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.access_control_map.insert(
            SPrincipal(get_global_super_admin_principal_id().0),
            vec![
                UserAccessRole::CanisterAdmin,
                UserAccessRole::CanisterController,
            ],
        );

        let principal_id = get_alice_principal_id().0;
        let user_roles = super::get_user_roles_impl(principal_id, &canister_data);

        assert_eq!(user_roles, vec![]);

        let principal_id = get_global_super_admin_principal_id().0;
        let user_roles = super::get_user_roles_impl(principal_id, &canister_data);
        assert_eq!(
            user_roles,
            vec![
                UserAccessRole::CanisterAdmin,
                UserAccessRole::CanisterController
            ]
        );
    }
}
