use candid::Principal;
use shared_utils::access_control::{self, UserAccessRole};

use crate::{data::CanisterData, CANISTER_DATA};

#[ic_cdk::query]
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
    access_control::get_roles_for_principal_id_v2(&canister_data.access_control_list, principal_id)
}

#[cfg(test)]
mod test {
    use shared_utils::access_control::UserAccessRole;
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id_v1, get_mock_user_alice_principal_id,
    };

    use crate::data::CanisterData;

    #[test]
    fn test_get_user_roles_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.access_control_list.insert(
            get_global_super_admin_principal_id_v1(),
            vec![
                UserAccessRole::CanisterAdmin,
                UserAccessRole::CanisterController,
            ],
        );

        let principal_id = get_mock_user_alice_principal_id();
        let user_roles = super::get_user_roles_impl(principal_id, &canister_data);

        assert_eq!(user_roles, vec![]);

        let principal_id = get_global_super_admin_principal_id_v1();
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
