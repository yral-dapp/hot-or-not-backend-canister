use std::collections::HashMap;

use candid::Principal;
use shared_utils::{
    access_control::UserAccessRole, common::types::known_principal::KnownPrincipalMap,
    constant::get_global_super_admin_principal_id_v1,
};

pub fn setup_initial_access_control_v1(
    user_id_access_control_map: &mut HashMap<Principal, Vec<UserAccessRole>>,
    known_principal_ids: &KnownPrincipalMap,
) {
    // * add global owner
    user_id_access_control_map.insert(
        get_global_super_admin_principal_id_v1(known_principal_ids.clone()),
        vec![
            UserAccessRole::CanisterController,
            UserAccessRole::CanisterAdmin,
        ],
    );
}

#[cfg(test)]
mod test {
    use shared_utils::common::types::known_principal::KnownPrincipalType;
    use test_utils::setup::test_constants::get_global_super_admin_principal_id_v1;

    use super::*;

    #[test]
    fn test_setup_initial_access_control_v1() {
        let mut user_id_access_control_map = HashMap::new();
        let mut known_principal_ids = KnownPrincipalMap::default();
        let global_super_admin = get_global_super_admin_principal_id_v1();
        known_principal_ids.insert(
            KnownPrincipalType::UserIdGlobalSuperAdmin,
            global_super_admin,
        );

        setup_initial_access_control_v1(&mut user_id_access_control_map, &known_principal_ids);

        assert_eq!(
            user_id_access_control_map.get(&global_super_admin),
            Some(&vec![
                UserAccessRole::CanisterController,
                UserAccessRole::CanisterAdmin,
            ])
        );
    }
}
